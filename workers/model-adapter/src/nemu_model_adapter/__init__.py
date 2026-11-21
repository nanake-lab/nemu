"""Local Qwen3.5 carrier and P5-42 resource consumer."""

from .carrier import LocalCarrier, Observation
from .cognition import local_result
from .native import Assets
from .resources import Publication, Unmodulated
from .speech import speech_result

__all__ = ["Assets", "LocalCarrier", "Observation", "Publication", "Unmodulated", "local_result", "speech_result"]
