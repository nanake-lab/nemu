"""Join independently tokenized segments using the official encoder's template."""
from __future__ import annotations
from typing import Protocol, cast
import unicodedata

class Encoder(Protocol):
    def encode_timestamp(self, text: str, language: str) -> tuple[list[str], str]: ...

class FixedSegments:
    def __init__(self, original: Encoder, full_text: str, segments: list[list[str]]) -> None:
        sentinel='NEMUCONTEXTUNIT'
        words,template=original.encode_timestamp(sentinel,'English')
        if words!=[sentinel] or template.count(sentinel)!=1:
            raise ValueError('Unexpected official timestamp encoding template')
        prefix,separator=template.split(sentinel)
        if not separator or any(not segment for segment in segments):
            raise ValueError('Empty segment or official separator')
        self.words=[word for segment in segments for word in segment]
        normalized=''.join(c for c in full_text if unicodedata.category(c)[0] in ('L','N') or c=="'")
        if ''.join(self.words)!=normalized:
            raise ValueError('Segment tokenization changed transcript characters')
        self.original=original
        self.full_text=full_text
        self.encoded=prefix+separator.join(self.words)+separator

    def encode_timestamp(self, text: str, language: str) -> tuple[list[str], str]:
        if text!=self.full_text:
            raise ValueError('Unexpected text for fixed segments')
        return self.words,self.encoded

    def __getattr__(self, name: str) -> object:
        # Keep the official parser/postprocessor unchanged.
        return cast(object,getattr(self.original,name))
