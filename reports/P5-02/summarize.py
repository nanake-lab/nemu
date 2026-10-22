"""Build acceptance evidence from the saved final cargo log; does not run tests."""
from __future__ import annotations

import hashlib
import json
import re
import subprocess
from pathlib import Path

BASE = Path(__file__).resolve().parent
REPO = BASE.parent.parent
LOG = BASE / "logs/10-final-workspace-tests.log"
text = LOG.read_text(encoding="utf-8-sig")
assert "test result: FAILED" not in text
assert "test result: ok. 23 passed; 0 failed" in text
assert "test result: ok. 2 passed; 0 failed" in text
cases = {
    "16.1": ["f161_exact_e1_e2_e2_survives_restart_without_second_ledger_charge", "duplicate_and_conflict_only_advance_processed_frontier"],
    "16.13": ["atomic_publication_and_append_only_history", "disk_full_rolls_back_ledger_and_all_state", "recovery_rejects_valid_but_mixed_head_and_missing_ledger"],
    "D10.1": ["d101_d102_stale_and_mutated_bindings_rejected", "all_binding_fields_are_immutable_after_dispatch", "later_ingress_cannot_overtake_uncommitted_input"],
    "D10.2": ["x03_real_process_exit_at_every_durability_boundary"],
    "X03": ["x03_real_process_exit_at_every_durability_boundary", "atomic_publication_and_append_only_history"],
    "AT01": ["at01_canonical_identity_and_superseded_writes"],
    "AT02": ["at02_corrupt_or_missing_object_never_publishes", "published_closure_damage_refuses_recovery"],
    "AT03": ["at03_pins_close_prepare_gc_publish_race"],
    "AT04": ["at04_single_writer_and_copy_cannot_resume_as_same_branch", "at04_old_database_in_original_location_cannot_refund_ledger"],
    "SYS22": ["sys22_capacity_and_busy_wait_are_bounded", "object_capacity_and_io_failures_never_acknowledge_d", "disk_full_rolls_back_ledger_and_all_state"],
    "I29": ["i29_all_six_cross_lineage_opens_rejected", "reader_and_capabilities_cannot_cross_stores_even_same_kind", "malformed_headers_and_existing_directories_never_initialize"],
    "AC2-22": ["i29_all_six_cross_lineage_opens_rejected", "reader_and_capabilities_cannot_cross_stores_even_same_kind"],
    "P5-32": ["r_index_reassembles_canonical_and_reuses_unchanged_tendencies", "p532_evidence_versions_and_ledger_prefix_are_complete"],
}
for point in ["G", "C", "L", "before", "sql", "after", "write", "file", "rename", "directory"]:
    expected = "new" if point in {"sql", "after"} else "old"
    assert f"X03 {point}: exit=86, recovered all-{expected}" in text
revision = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=REPO, text=True).strip()
paths = ["core/crates/core-store", "core/Cargo.lock", ".github/workflows/windows.yml"]
source = subprocess.check_output(["git", "ls-tree", "-r", "HEAD", "--", *paths], cwd=REPO, text=True)
manifest = {"revision": revision, "source_tree_entries": source.splitlines(), "source_tree_sha256": hashlib.sha256(source.encode()).hexdigest(),
            "log": str(LOG.relative_to(BASE)).replace("\\", "/"), "log_sha256_lf_utf8": hashlib.sha256(text.encode()).hexdigest(),
            "command": "cargo test --workspace --locked -- --test-threads=1 --nocapture", "exit_code": 0,
            "scope": "W02 storage only; synthetic TemporaryRoot; no production data, GPU, real power-cut or operator validity claim",
            "suite_summaries": re.findall(r"test result: ok\..*", text)}
(BASE / "final-test-manifest.json").write_text(json.dumps(manifest, ensure_ascii=False, indent=2)+"\n", encoding="utf-8")
for key, names in cases.items():
    for name in names:
        assert re.search(r"test tests::\w+::" + re.escape(name) + r" \.\.\.", text), name
    folder = BASE / key
    folder.mkdir(exist_ok=True)
    result = {"id": key, "result": "passed", "scope": "storage subset", "source_revision": revision, "tests": names,
              "evidence": "../logs/10-final-workspace-tests.log", "manifest": "../final-test-manifest.json"}
    (folder / "result.json").write_text(json.dumps(result, ensure_ascii=False, indent=2)+"\n", encoding="utf-8")
    (folder / "README.md").write_text(f"# {key} — W02 存储验收\n\n合成夹具，仅新建临时目录。构造、正例/拒绝条件见上述测试源码与规格对照表。\n\n"
        + "\n".join(f"- `{name}`" for name in names) + "\n\n结果见 result.json，原始日志和源文件指纹见上级目录。\n", encoding="utf-8")
print(json.dumps({"revision": revision, "acceptance_groups": len(cases), "store_tests": 23, "compile_fail_doctests": 2}))
