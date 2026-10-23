"""Concurrency, bounded cancellation, failure propagation and graph restoration."""
from contextlib import nullcontext
from concurrent.futures import ThreadPoolExecutor
from threading import Event
from types import SimpleNamespace
from typing import Any
import pytest
from nemu_voice_sidecar.decode_worker import DecodeWorker
from nemu_voice_sidecar.cuda_graph import ForwardGraphs
from nemu_voice_sidecar.profiling import profile_forwards


class Stream:
    def wait_stream(self, other: object) -> None:
        pass
    def wait_event(self, event: object) -> None:
        pass
    def synchronize(self) -> None:
        pass


class Ready:
    def record(self) -> None:
        pass


def fake_torch() -> SimpleNamespace:
    return SimpleNamespace(inference_mode=nullcontext,
        cuda=SimpleNamespace(Stream=Stream, Event=Ready, stream=lambda s: nullcontext()))


def test_producer_can_continue_while_decode_waits_and_cancel_discards_queue() -> None:
    entered, release, cancel = Event(), Event(), Event()
    calls = []
    def decode(job: object) -> None:
        calls.append(job)
        entered.set()
        assert release.wait(2)
    worker = DecodeWorker(fake_torch(), decode, cancel)
    worker.submit(1)
    assert entered.wait(2)
    worker.submit(2)  # Producer proceeds while decode is blocked.
    cancel.set()
    release.set()
    worker.finish()
    assert calls == [1] and worker.thread is not None and not worker.thread.is_alive()


def test_worker_failure_is_not_reported_as_success() -> None:
    def decode(job: object) -> None:
        raise RuntimeError("decoder failed")
    worker = DecodeWorker(fake_torch(), decode, Event())
    worker.submit(1)
    with pytest.raises(RuntimeError, match="decoder failed"):
        worker.finish()


def test_requests_reuse_one_native_thread_and_release_request_callbacks() -> None:
    import gc
    import weakref
    from threading import get_ident

    threads: set[int] = set()

    class Callback:
        def __call__(self, job: object) -> None:
            threads.add(get_ident())

    refs = []
    with ThreadPoolExecutor(max_workers=1) as executor:
        for _ in range(100):
            callback = Callback()
            refs.append(weakref.ref(callback))
            worker = DecodeWorker(fake_torch(), callback, Event(), stream=Stream(), executor=executor)
            worker.submit(1)
            worker.finish()
            assert worker.future is not None and worker.future.done()
            del callback, worker
        gc.collect()
        assert len(threads) == 1
        assert all(ref() is None for ref in refs)


def test_persistent_thread_recovers_after_cancel_and_decoder_failure() -> None:
    calls: list[object] = []
    with ThreadPoolExecutor(max_workers=1) as executor:
        cancel = Event()
        cancel.set()
        worker = DecodeWorker(fake_torch(), calls.append, cancel, executor=executor)
        worker.submit(1)
        worker.finish()
        def fail(job: object) -> None:
            raise ValueError("decode failure")
        worker = DecodeWorker(fake_torch(), fail, Event(), executor=executor)
        worker.submit(1)
        with pytest.raises(ValueError, match="decode failure"):
            worker.finish()
        worker = DecodeWorker(fake_torch(), calls.append, Event(), executor=executor)
        worker.submit(2)
        worker.finish()
    assert calls == [2]


def test_graph_model_patch_is_scoped_even_on_generation_failure() -> None:
    graphs = object.__new__(ForwardGraphs)
    graphs.inner = SimpleNamespace(forward="upstream talker")
    graphs.predictor = SimpleNamespace(forward="upstream predictor")
    graphs.original_talker, graphs.original_predictor = graphs.inner.forward, graphs.predictor.forward
    with pytest.raises(ValueError):
        with graphs.installed():
            assert graphs.inner.forward == graphs.talker_forward
            raise ValueError("request failed")
    assert graphs.inner.forward == "upstream talker"
    assert graphs.predictor.forward == "upstream predictor"


def test_repeated_graph_and_profile_contexts_do_not_chain_callbacks() -> None:
    def forward(**kwargs: Any) -> str:
        return "upstream"
    graphs = object.__new__(ForwardGraphs)
    graphs.inner = SimpleNamespace(forward=forward)
    graphs.predictor = SimpleNamespace(forward=forward)
    graphs.original_talker = graphs.original_predictor = forward
    # A CPU stand-in for a graph replay; exercise the real install/restore logic.
    graphs.talker_forward = forward  # type: ignore[method-assign]
    graphs.predictor_forward = forward  # type: ignore[method-assign]
    tokenizer = SimpleNamespace(decode=forward)
    model = SimpleNamespace(model=SimpleNamespace(talker=SimpleNamespace(
        model=graphs.inner, code_predictor=graphs.predictor), speech_tokenizer=tokenizer))
    trace: Any = SimpleNamespace(folder="enabled", span=lambda *a: nullcontext())
    for _ in range(1000):
        with graphs.installed(), profile_forwards(model, trace):
            assert graphs.inner.forward(inputs_embeds=None) == "upstream"
            assert graphs.predictor.forward() == "upstream"
            assert tokenizer.decode() == "upstream"
        assert graphs.inner.forward is forward
        assert graphs.predictor.forward is forward
        assert tokenizer.decode is forward


@pytest.mark.parametrize("frames", [0, 1, 17, 48])
def test_graph_capture_budget_rejects_unsupported_counts(frames: int) -> None:
    with pytest.raises(ValueError, match="INVALID_GRAPH_FRAME_BUDGET"):
        ForwardGraphs(None, None, frames_per_request=frames)


@pytest.mark.parametrize("position", [125, 256])
def test_transition_preserves_attention_mask_at_static_cache_boundary(
    monkeypatch: pytest.MonkeyPatch, position: int,
) -> None:
    class Tensor:
        ndim = 4
        device = "cuda:0"
        dtype = "bf16"
        def __init__(self, shape: tuple[int, ...]) -> None:
            self.shape = shape
        def __getitem__(self, key: tuple[Any, ...]) -> Any:
            shape = list(self.shape)
            axis = -1 if len(key) == 2 else -2
            shape[axis] = min(shape[axis], key[axis].stop or shape[axis])
            return Tensor(tuple(shape))
        def contiguous(self) -> Any:
            return self
    class Cache:
        def update(self, keys: Any, values: Any, index: int) -> None:
            assert keys.shape[-2] == position and values.shape[-2] == position
    monkeypatch.setattr("nemu_voice_sidecar.cuda_graph.importlib.import_module",
                        lambda name: SimpleNamespace(DynamicCache=Cache))
    def cat(tensors: tuple[Any, ...], dim: int) -> Any:
        assert dim == -1
        shape = list(tensors[0].shape)
        shape[-1] = sum(t.shape[-1] for t in tensors)
        return Tensor(tuple(shape))
    graph = object.__new__(ForwardGraphs)
    graph.eager_talker = False
    graph.next_position = graph.request_graph_end = position
    graph.talker_cache = SimpleNamespace(layers=[SimpleNamespace(
        keys=Tensor((1, 2, 256, 8)), values=Tensor((1, 2, 256, 8)))])
    graph.torch = SimpleNamespace(zeros=lambda shape, **kwargs: Tensor(shape), cat=cat)
    def upstream(**kwargs: Any) -> Any:
        assert isinstance(kwargs["past_key_values"], Cache)
        assert kwargs["attention_mask"].shape[-1] == position+1
        return "continued upstream"
    graph.original_talker = upstream
    assert graph.talker_forward(inputs_embeds=Tensor((1, 1, 1024)),
        attention_mask=Tensor((1, 1, 1, 256))) == "continued upstream"
    assert graph.eager_talker
