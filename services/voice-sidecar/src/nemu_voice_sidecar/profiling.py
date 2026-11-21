"""Request-scoped forward timers separate talker backbone from code predictor."""
from __future__ import annotations
from collections.abc import Iterator
from contextlib import contextmanager
from typing import Any
from .timing import Trace


@contextmanager
def profile_forwards(model: Any, trace: Trace) -> Iterator[None]:
    if not trace.folder:
        yield
        return
    inner = model.model.talker.model
    predictor = model.model.talker.code_predictor
    talker_forward, predictor_forward = inner.forward, predictor.forward
    tokenizer = model.model.speech_tokenizer
    original_decode = tokenizer.decode
    decodes = 0
    frame = 0
    code = 0

    def talker(*args: Any, **kwargs: Any) -> Any:
        nonlocal frame
        embeds = kwargs.get("inputs_embeds")
        prefill = embeds is not None and embeds.shape[1] > 1
        with trace.span("prefill" if prefill else "talker", frame):
            result = talker_forward(*args, **kwargs)
        if not prefill:
            frame += 1
        return result

    def predict(*args: Any, **kwargs: Any) -> Any:
        nonlocal code
        with trace.span("code_predictor", code):
            result = predictor_forward(*args, **kwargs)
        code += 1
        return result

    def decode(*args: Any, **kwargs: Any) -> Any:
        nonlocal decodes
        index = decodes
        decodes += 1
        with trace.span("tokenizer_decode", index):
            return original_decode(*args, **kwargs)

    tokenizer.decode = decode
    inner.forward, predictor.forward = talker, predict
    try:
        yield
    finally:
        tokenizer.decode = original_decode
        inner.forward, predictor.forward = talker_forward, predictor_forward
