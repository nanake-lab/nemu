import nemu_drive


def test_package_import() -> None:
    assert nemu_drive.__doc__ is not None
