"""Verify saved review regression evidence; never opens a storage database."""
from __future__ import annotations

import hashlib
import json
import re
import subprocess
from pathlib import Path

BASE = Path(__file__).resolve().parent
REPO = BASE.parents[2]
LOG = BASE.parent / "logs/16-review-final-workspace.log"
META = LOG.with_suffix(".json")
REVISION = "1e1cc584dba4abaa91be11f2296195a2f320e303"
raw = LOG.read_bytes()
text = raw.decode("utf-8-sig").replace("\r\n", "\n")
meta = json.loads(META.read_text(encoding="utf-8-sig"))
assert meta["exitCode"] == 0, meta
assert meta["sourceRevision"] == REVISION, meta
# Git stores text evidence with LF; the collector also retains the original
# Windows byte hash as provenance. Validate the portable normalized form.
assert meta["lfUtf8LogSha256"].lower() == hashlib.sha256(text.encode()).hexdigest()
assert "test result: FAILED" not in text
assert "test result: ok. 31 passed; 0 failed" in text
assert "test result: ok. 2 passed; 0 failed" in text

objects = ["ObjectWritten", "FileSynced", "Renamed", "DirectorySynced"]
receipts = [
    "ReceiptWritten", "ReceiptFileSyncBefore", "ReceiptFileSynced",
    "ReceiptRenameBefore", "ReceiptRenamed", "ReceiptTargetSyncBefore",
    "ReceiptTargetSynced", "ReceiptDirectorySyncBefore", "ReceiptDirectorySynced",
]
expected: dict[str, tuple[int, int]] = {}
for point in ["GStaged", "CStaged", "LedgerWritten", "BeforePublish", "SqlCommitted", "AfterPublish"]:
    expected[f"commit/{point}"] = (2 if point in {"SqlCommitted", "AfterPublish"} else 1, 2)
for point in objects + ["IngressBeforeSql", "IngressSqlCommitted", "IngressAcknowledged"]:
    expected[f"ingress/{point}"] = (1, 2 if point in {"IngressSqlCommitted", "IngressAcknowledged"} else 1)
for operation in ["ingress", "commit"]:
    for repair in ["false", "true"]:
        for point in receipts:
            expected[f"{operation}/repair={repair}/{point}"] = (2 if operation == "commit" else 1, 2)
for point in objects:
    for operation, tag, nth in [
        ("r", "echo.tendency-v2", 2), ("r", "echo.r-snapshot-v2", 1),
        ("manifest-second", "echo.transaction-v2", 2),
    ]:
        expected[f"{operation}/{tag}/{nth}/{point}"] = (1, 2)
for point in ["ReuseFileSyncBefore", "ReuseFileSynced", "ReuseDirectorySyncBefore", "ReuseDirectorySynced"]:
    expected[f"reuse/FastStateSnapshotV2/2/{point}"] = (1, 1)
for point, nth in [("GcDeleted", 1), ("GcDeleted", 2), ("GcDirectorySyncBefore", 1), ("GcDirectorySynced", 1)]:
    expected[f"gc/{point}/{nth}"] = (1, 1)
matches = re.findall(
    r"^X03 ([^:]+): exit=86; state=(\d+); D=(\d+); closure/L/H/concepts/accepted/receipts/reopen/GC verified$",
    text, re.MULTILINE,
)
actual = {label: (int(state), int(ingress)) for label, state, ingress in matches}
assert len(matches) == len(actual) == 69, matches
assert actual == expected, (actual.keys() ^ expected.keys(), actual)

initialization: dict[str, str] = {}
for point in [
    "InitRootCreated", "InitRootSynced", "InitObjectsCreated", "InitObjectsSynced",
    "InitSqlCommitted", "InitDbDirectorySynced", "ReceiptRootCreated", "ReceiptKindCreated",
    "ReceiptKindSynced", "ReceiptRootSynced", "ReceiptParentSynced",
]:
    for nth in ([1, 2] if point in {"ReceiptKindCreated", "ReceiptKindSynced"} else [1]):
        complete = point in {"ReceiptRootSynced", "ReceiptParentSynced"} or (
            point in {"ReceiptKindCreated", "ReceiptKindSynced"} and nth == 2
        )
        initialization[f"{point}/{nth}"] = "verified empty store" if complete else "partial initialization refused"
init_matches = re.findall(r"^INIT ([^:]+): exit=86; (.+)$", text, re.MULTILINE)
assert len(init_matches) == len(dict(init_matches)) == 13
assert dict(init_matches) == initialization
injections = re.findall(r"^W02_INJECT .+$", text, re.MULTILINE)
assert len(injections) == 100, len(injections)  # 82 cases plus 18 interrupted-repair preludes.

cases = {
    "16.1": ["f161_exact_e1_e2_e2_survives_restart_without_second_ledger_charge", "duplicate_and_conflict_only_advance_processed_frontier"],
    "16.13": ["atomic_publication_and_append_only_history", "disk_full_rolls_back_ledger_and_all_state", "recovery_rejects_valid_but_mixed_head_and_missing_ledger"],
    "D10.1": ["d101_d102_stale_and_mutated_bindings_rejected", "all_binding_fields_are_immutable_after_dispatch", "later_ingress_cannot_overtake_uncommitted_input"],
    "D10.2": ["x03_ingress_and_transaction_publication_boundaries", "x03_receipt_publish_and_repair_can_crash_again", "x03_targeted_r_segments_manifest_and_reused_objects"],
    "X03": ["x03_ingress_and_transaction_publication_boundaries", "x03_receipt_publish_and_repair_can_crash_again", "x03_targeted_r_segments_manifest_and_reused_objects", "atomic_publication_and_append_only_history"],
    "AT01": ["at01_canonical_identity_and_superseded_writes"],
    "AT02": ["at02_corrupt_or_missing_object_never_publishes", "published_closure_damage_refuses_recovery", "recovery_rejects_valid_but_mixed_head_and_missing_ledger", "publication_primitive_never_replaces_existing_target"],
    "AT03": ["at03_pins_close_prepare_gc_publish_race", "initialization_and_gc_crash_boundaries_have_separate_expectations"],
    "AT04": ["at04_single_writer_and_copy_cannot_resume_as_same_branch", "at04_old_database_in_original_location_cannot_refund_ledger", "reader_receipts_precede_sql_snapshot_under_controlled_appends", "live_readers_open_during_ingress_without_false_rollback"],
    "SYS22": ["sys22_capacity_and_busy_wait_are_bounded", "object_capacity_and_io_failures_never_acknowledge_d", "disk_full_rolls_back_ledger_and_all_state"],
    "I29": ["i29_all_six_cross_lineage_opens_rejected", "reader_and_capabilities_cannot_cross_stores_even_same_kind", "malformed_headers_and_existing_directories_never_initialize", "stray_environment_or_missing_permit_never_touches_target"],
    "AC2-22": ["i29_all_six_cross_lineage_opens_rejected", "reader_and_capabilities_cannot_cross_stores_even_same_kind"],
    "P5-32": ["r_index_reassembles_canonical_and_reuses_unchanged_tendencies", "p532_evidence_versions_and_ledger_prefix_are_complete", "segmented_r_commit_retains_both_tendencies_through_reopen_and_gc"],
}
for names in cases.values():
    for name in names:
        assert re.search(r"test tests::\w+::" + re.escape(name) + r" \.\.\.", text), name
summaries = re.findall(r"test result: ok\..*", text)
passed = sum(int(n) for n in re.findall(r"test result: ok\. (\d+) passed; 0 failed", text))
assert passed == 85, passed
paths = ["core", ".github/workflows/ci.yml", ".github/workflows/windows.yml"]
source = subprocess.check_output(["git", "ls-tree", "-r", REVISION, "--", *paths], cwd=REPO, text=True)
current = subprocess.check_output(["git", "ls-tree", "-r", "HEAD", "--", *paths], cwd=REPO, text=True)
assert source == current, "HEAD source differs from tested revision"
dirty = subprocess.check_output(["git", "diff", "HEAD", "--", *paths], cwd=REPO, text=True)
assert not dirty, "uncommitted changes to tested source"
manifest = {
    "sourceRevision": REVISION, "sourceTreeEntries": source.splitlines(),
    "sourceTreeSha256": hashlib.sha256(source.encode()).hexdigest(),
    "log": "../logs/16-review-final-workspace.log", "metadata": "../logs/16-review-final-workspace.json",
    "rawLogSha256AtCollection": meta["rawLogSha256"].lower(),
    "lfUtf8LogSha256": hashlib.sha256(text.encode()).hexdigest(),
    "command": meta["command"], "exitCode": meta["exitCode"],
    "startTime": meta["startTime"], "endTime": meta["endTime"],
    "passed": passed, "storeUnitTests": 31, "storeCompileFailDoctests": 2,
    "recoveryCases": [{"case": label, "state": state, "rawIngress": ingress} for label, (state, ingress) in actual.items()],
    "initializationCases": initialization, "injectionCount": len(injections), "suiteSummaries": summaries,
    "scope": "W02 synthetic TemporaryRoot only; process exit, not physical power-cut or device certification",
}
(BASE / "final-test-manifest.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
for key, names in cases.items():
    folder = BASE / key
    folder.mkdir(exist_ok=True)
    result = {"id": key, "result": "passed", "scope": "storage subset", "sourceRevision": REVISION,
              "tests": names, "evidence": "../../logs/16-review-final-workspace.log", "manifest": "../final-test-manifest.json"}
    (folder / "result.json").write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"sourceRevision": REVISION, "passed": passed, "acceptanceGroups": len(cases), "cases": 82, "injections": len(injections)}))
