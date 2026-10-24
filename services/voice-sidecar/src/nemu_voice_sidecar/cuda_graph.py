"""Pinned 0.1.1/4.57.3 forward graphs; HF generation and sampling stay upstream.

Own implementation using torch CUDAGraph and transformers StaticCache. No kernel,
precision, sampling, vocabulary head or text conditioning is replaced.
"""
from __future__ import annotations
from contextlib import contextmanager
from collections.abc import Iterator
import importlib
import importlib.metadata
from typing import Any


class Step:
    def __init__(self, torch: Any, forward: Any, kwargs: dict[str, Any], pool: Any = None, stream: Any = None) -> None:
        self.torch = torch
        self.kwargs = kwargs
        self.graph = torch.cuda.CUDAGraph()
        stream = stream if stream is not None else torch.cuda.Stream()
        stream.wait_stream(torch.cuda.current_stream())
        with torch.cuda.stream(stream):
            for _ in range(3):
                forward(**kwargs)
            stream.synchronize()
            with torch.cuda.graph(self.graph, stream=stream, pool=pool):
                self.output = forward(**kwargs)
        torch.cuda.current_stream().wait_stream(stream)
        stream.synchronize()

    def run(self) -> Any:
        self.graph.replay()
        return self.output


class ForwardGraphs:
    def __init__(self, model: Any, torch: Any, max_length: int = 256, generation_length: int = 512,
                 frames_per_request: int = 16) -> None:
        if frames_per_request not in (4, 8, 16):
            raise ValueError("INVALID_GRAPH_FRAME_BUDGET")
        if any(importlib.metadata.version(package) != expected for package, expected in (
            ("qwen-tts", "0.1.1"), ("transformers", "4.57.3"), ("torch", "2.10.0+cu128"))):
            raise RuntimeError("GRAPH_API_VERSION_MISMATCH")
        self.torch = torch
        self.talker = model.model.talker
        self.predictor = self.talker.code_predictor
        self.inner = self.talker.model
        self.original_talker = self.inner.forward
        self.original_predictor = self.predictor.forward
        self.graph_budget = 48
        self.frames_per_request = frames_per_request
        self.request_graph_end = 0
        self.length = max_length
        self.generation_length = generation_length
        self.visible_length = max_length
        self.eager_talker = False
        self.trace: Any = None
        self.talker_cache = self.cache(self.inner.config, max_length)
        self.predictor_cache = self.cache(self.predictor.config, 17)
        self.predictor_visible = 17
        hidden = self.inner.config.hidden_size
        opts = {"dtype": torch.bfloat16, "device": "cuda:0"}
        self.embed = torch.zeros(1, 1, hidden, **opts)
        self.position = torch.zeros(1, dtype=torch.long, device="cuda:0")
        self.rope = torch.zeros(3, 1, 1, dtype=torch.long, device="cuda:0")
        self.mask = torch.zeros(1, 1, 1, max_length, **opts)
        self.keys = torch.arange(max_length, device="cuda:0")
        # Match upstream attention geometry exactly instead of padding its softmax
        # to 1024 keys (that changes greedy codec decisions in bf16).
        original_update = self.talker_cache.update

        def update(*args: Any, **kwargs: Any) -> Any:
            k, v = original_update(*args, **kwargs)
            return k[..., :self.visible_length, :].contiguous(), v[..., :self.visible_length, :].contiguous()

        self.talker_cache.update = update
        self.pool = torch.cuda.graph_pool_handle()
        self.capture_stream = torch.cuda.Stream()
        self.talker_steps: dict[int, Step] = {}
        original_predictor_update = self.predictor_cache.update

        def predictor_update(*args: Any, **kwargs: Any) -> Any:
            k, v = original_predictor_update(*args, **kwargs)
            return k[..., :self.predictor_visible, :].contiguous(), v[..., :self.predictor_visible, :].contiguous()

        self.predictor_cache.update = predictor_update
        self.predictor_steps = []
        for index in range(15):
            self.predictor_visible = index+2
            pos = torch.arange(2, device="cuda:0") if index == 0 else torch.tensor([index+1], device="cuda:0")
            allowed = torch.arange(17, device="cuda:0")[None, :self.predictor_visible] <= pos[:, None]
            mask = torch.where(allowed, 0., torch.finfo(torch.bfloat16).min).to(torch.bfloat16)[None, None]
            kwargs: dict[str, Any] = {
                "inputs_embeds": torch.zeros(1, 2, hidden, **opts) if index == 0 else None,
                "input_ids": None if index == 0 else torch.zeros(1, 1, dtype=torch.long, device="cuda:0"),
                "past_key_values": self.predictor_cache, "cache_position": pos,
                "position_ids": pos[None, :], "attention_mask": {"full_attention": mask},
                "generation_steps": index, "use_cache": True, "output_hidden_states": True}
            self.predictor_steps.append(Step(torch, self.original_predictor, kwargs,
                                             pool=self.pool, stream=self.capture_stream))
        self.talker_cache.reset()
        self.predictor_cache.reset()
        self.next_position = 0

    def cache(self, config: Any, length: int) -> Any:
        api = importlib.import_module("transformers")
        cache = api.StaticCache(config=config, max_cache_len=length)
        cache.early_initialization(1, config.num_key_value_heads,
            getattr(config, "head_dim", config.hidden_size // config.num_attention_heads),
            self.torch.bfloat16, self.torch.device("cuda:0"))
        return cache

    def talker_forward(self, *args: Any, **kwargs: Any) -> Any:
        if args:
            raise RuntimeError("GRAPH_POSITIONAL_INPUT_UNSUPPORTED")
        embeds = kwargs["inputs_embeds"]
        if embeds.shape[1] > 1:
            n = embeds.shape[1]
            needed = set(range(n, min(n+self.frames_per_request, self.length)))
            self.eager_talker = n + self.frames_per_request > self.length or len(set(self.talker_steps) | needed) > self.graph_budget
            if self.eager_talker:
                return self.original_talker(**kwargs)
            if self.trace:
                with self.trace.span("upstream_prefill"):
                    out = self.original_talker(**kwargs)
            else:
                out = self.original_talker(**kwargs)
            self.talker_cache.reset()
            positions = self.torch.arange(n, device=embeds.device)
            for index, layer in enumerate(out.past_key_values.layers):
                self.talker_cache.update(layer.keys, layer.values, index, {"cache_position": positions})
            if self.trace:
                with self.trace.span("graph_prepare", n):
                    self.prepare_talker(n)
            else:
                self.prepare_talker(n)
            self.next_position = n
            out.past_key_values = self.talker_cache
            return out
        if self.eager_talker:
            return self.original_talker(**kwargs)
        pos = self.next_position
        if pos >= self.request_graph_end:
            # Bound graph metadata/buffers and preserve all remaining codec frames.
            api = importlib.import_module("transformers")
            dynamic = api.DynamicCache()
            for index, layer in enumerate(self.talker_cache.layers):
                dynamic.update(layer.keys[..., :pos, :].contiguous(),
                               layer.values[..., :pos, :].contiguous(), index)
            kwargs["past_key_values"] = dynamic
            attention = kwargs.get("attention_mask")
            if attention is not None and attention.ndim == 4:
                current = attention[..., :pos+1]
                if current.shape[-1] < pos+1:
                    extra = self.torch.zeros((*current.shape[:-1], pos+1-current.shape[-1]),
                                             device=current.device, dtype=current.dtype)
                    current = self.torch.cat((current, extra), dim=-1)
                kwargs["attention_mask"] = current
            self.eager_talker = True
            return self.original_talker(**kwargs)
        if pos >= self.length:
            raise RuntimeError("GRAPH_CONTEXT_LIMIT")
        self.embed.copy_(embeds)
        self.position.copy_(kwargs["cache_position"])
        self.rope.copy_(kwargs["position_ids"])
        allowed = self.keys <= pos
        window = self.inner.config.sliding_window
        if window is not None:
            allowed &= self.keys > pos-window
        mask = kwargs.get("attention_mask")
        if mask is not None and mask.ndim == 4:
            # HF prepares an additive static-cache mask before calling the model.
            self.mask.copy_(mask)
        else:
            if mask is not None:
                padding = self.torch.ones(self.length, dtype=self.torch.bool, device=embeds.device)
                padding[:mask.shape[-1]].copy_(mask[0].bool())
                allowed &= padding
            self.mask.copy_(self.torch.where(allowed, 0., self.torch.finfo(embeds.dtype).min)[None, None, None])
        self.next_position += 1
        return self.talker_steps[pos].run()

    def prepare_talker(self, start: int) -> None:
        # Capture before emitting any codec jobs. Graphs share a pool; each output
        # is consumed before the next talker replay. External cache/input buffers
        # have independent storage. The per-position graph keeps dynamic-cache
        # matmul/softmax shapes, while KV storage addresses remain fixed.
        self.request_graph_end = min(start+self.frames_per_request, self.length)
        for position in range(start, self.request_graph_end):
            if position in self.talker_steps:
                continue
            self.visible_length = position+1
            self.position.fill_(position)
            self.rope.fill_(position)
            self.mask.zero_()
            step = Step(self.torch, self.original_talker, {
                "inputs_embeds": self.embed, "past_key_values": self.talker_cache,
                "cache_position": self.position, "position_ids": self.rope,
                "attention_mask": self.mask[..., :position+1], "use_cache": True,
                "output_hidden_states": False}, pool=self.pool, stream=self.capture_stream)
            # Upstream voice-clone only consumes the last backbone hidden state.
            # Keep that exact tensor, without retaining every layer in every graph.
            step.output.hidden_states = (step.output.last_hidden_state,)
            self.talker_steps[position] = step

    def predictor_forward(self, *args: Any, **kwargs: Any) -> Any:
        if args:
            raise RuntimeError("GRAPH_POSITIONAL_INPUT_UNSUPPORTED")
        embeds = kwargs.get("inputs_embeds")
        prefill = embeds is not None and embeds.shape[1] > 1
        index = 0 if prefill else int(kwargs["generation_steps"])
        if prefill:
            self.predictor_cache.reset()
        step = self.predictor_steps[index]
        if prefill:
            step.kwargs["inputs_embeds"].copy_(embeds)
        else:
            step.kwargs["input_ids"].copy_(kwargs["input_ids"])
        return step.run()

    @contextmanager
    def installed(self, *, talker: bool = True, predictor: bool = True) -> Iterator[None]:
        # Only this model instance, under the service's exclusive inference lane.
        if talker:
            self.inner.forward = self.talker_forward
        if predictor:
            self.predictor.forward = self.predictor_forward
        try:
            yield
        finally:
            self.inner.forward = self.original_talker
            self.predictor.forward = self.original_predictor
