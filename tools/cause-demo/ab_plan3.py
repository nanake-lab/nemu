from pathlib import Path

from ab_plan import main

CELLS = tuple((question, method) for question in ("Q1", "Q2", "Q3", "Q4") for method in ("C", "E"))
LOG_DIR = Path(r"D:\Dev\cache\cause-demo\ab-plan3")
SMOKE_ROUTES = {("Q2", "E"): "slow", ("Q4", "E"): "fast"}


if __name__ == "__main__":
    raise SystemExit(main(CELLS, log_dir=LOG_DIR, smoke_routes=SMOKE_ROUTES))
