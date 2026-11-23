from pathlib import Path

from ab_plan import main

CELLS = (
    ("Q1", "A"), ("Q1", "B"), ("Q1", "C"), ("Q1", "D"),
    ("Q2", "A"), ("Q2", "B"), ("Q2", "C"),
    ("Q3", "A"), ("Q3", "B"), ("Q3", "C"),
)
LOG_DIR = Path(r"D:\Dev\cache\cause-demo\ab-plan2")


if __name__ == "__main__":
    raise SystemExit(main(CELLS, ("Q1", "C"), LOG_DIR))
