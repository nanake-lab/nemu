"""Lightweight SQL probes only; this does not run or validate the Rust Store."""
from pathlib import Path
import re
import sqlite3

ROOT = Path(__file__).resolve().parents[4]
SOURCE = ROOT / "core/crates/core-store/src"
queries = (SOURCE / "queries.rs").read_text(encoding="utf-8")
validation = (SOURCE / "r_validation.rs").read_text(encoding="utf-8")
replay = (SOURCE / "r_replay.rs").read_text(encoding="utf-8")


def sql(source: str, prefix: str) -> str:
    return next(q for q in re.findall(r'prepare\("([^"\n]+)"\)', source) if q.startswith(prefix))


db = sqlite3.connect(":memory:")
db.executescript("""
CREATE TABLE commits(id TEXT PRIMARY KEY, position INTEGER, decision TEXT);
CREATE TABLE evidence_groups(commit_id TEXT, group_id TEXT, reference TEXT);
CREATE TABLE evidence_index(commit_id TEXT, tendency_id TEXT, group_id TEXT, reference TEXT);
CREATE TABLE snapshots(commit_id TEXT, reference TEXT);
INSERT INTO commits VALUES ('old',0,'accepted'),('filtered',1,'filtered'),('new',2,'accepted'),('late',3,'accepted');
INSERT INTO snapshots VALUES ('old','transaction-v2'),('new','transaction-v3'),('late','correction');
INSERT INTO evidence_groups VALUES ('old','g','v2'),('new','g','v3-empty'),('late','g','v3-other'),('old','h','h-v2'),('new','h','h-v3');
INSERT INTO evidence_index VALUES ('old','t','g','v2'),('late','other','g','v3-other'),('old','t','h','h-v2'),('new','t','h','h-v3');
""")
query = sql(queries, "WITH latest AS")
assert db.execute(query, (0, "t", 11)).fetchall() == [("v2",), ("h-v2",)]
assert db.execute(query, (2, "t", 11)).fetchall() == [("h-v3",)]
assert db.execute(query, (3, "t", 11)).fetchall() == [("h-v3",)]
assert db.execute(query, (3, "other", 11)).fetchall() == [("v3-other",)]
assert len(db.execute(query, (0, "t", 2)).fetchall()) == 2  # limit+1 exposes overflow
latest = sql(validation, "WITH latest AS")
assert db.execute(latest, (10,)).fetchall() == [("v3-other",), ("h-v3",)]
history = sql(replay, "SELECT reference FROM snapshots")
assert db.execute(history).fetchall() == [("transaction-v2",), ("transaction-v3",), ("correction",)]
print(f"SQLite {sqlite3.sqlite_version}; 7 SQL probes passed using source-extracted queries")
print("Scope: latest-before-association, empty successor, reassignment, limit+1, complete accepted ordering")
print("Not run: Rust compilation, core-store tests, G replay, SYS17 or crash acceptance")
