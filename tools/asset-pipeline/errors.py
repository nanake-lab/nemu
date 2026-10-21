from typing import NoReturn


class BuildError(ValueError):
    """An actionable step/object/expected/actual diagnostic."""


def fail(step: str, obj: str, expected: object, actual: object) -> NoReturn:
    raise BuildError(f"step={step} object={obj} expected={expected} actual={actual}")


