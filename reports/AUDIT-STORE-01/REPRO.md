# AUDIT-STORE-01 复现说明

基线：`ce115abe6ced3ec8cc301bea2a290f47ace4807a`。所有命令在该基线的仓库根目录执行，测试源码仅写到 D 盘独立探针目录；不修改仓库源代码。仅 `TemporaryRoot::new()` 创建的合成目录会被访问。两个正常故障反例应失败；两个标为 boundary/known 的诊断用断言确认所描述的漏洞表现，诊断通过不等于产品修复。

## 1. 构建独立探针

下列 PowerShell 复用当前仓库 W02 测试的合成夹具，只把 helper 对私有字段的读取替换为对应公开 API。没有移除被测边界的断言，也没有改动 core-store。依赖均为仓库已有版本。

```powershell
$repo = (Resolve-Path .).Path.Replace('\', '/')
$probe = 'D:/Dev/cache/AUDIT-STORE-01-review/probe'
New-Item -ItemType Directory -Force (Join-Path $probe 'src') | Out-Null
@"
[package]
name = "audit-store-01-probe"
version = "0.1.0"
edition = "2024"
[dependencies]
core-store = { path = "$repo/core/crates/core-store" }
core-types = { path = "$repo/core/crates/core-types" }
serde_json = "=1.0.154"
rusqlite = { version = "=0.40.2", default-features = false, features = ["bundled", "limits"] }
"@ | Set-Content -Encoding utf8 (Join-Path $probe 'Cargo.toml')
$source = Get-Content -Raw core/crates/core-store/src/tests/mod.rs
$source = $source.Replace('use super::*;', 'use core_store::*;')
$source = [regex]::Replace($source, '(?m)^mod \w+;\r?\n', '')
$source = $source.Replace('../../../../contracts/golden/legal.json', "$repo/core/contracts/golden/legal.json")
$source = $source.Replace('store.branch.clone()', '"test-branch".into()')
$source = [regex]::Replace($source, '(FastStateSnapshotV2|RSnapshotV3|ThetaSnapshotV2)::from_canonical\(\s*&store.objects.read\(&old\.(staged_\w+)\).unwrap\(\),\s*config\(\).encoding,\s*\)\s*.unwrap\(\)', 'store.reader().unwrap().read::<$1>(&old.$2).unwrap()')
$source = $source.Replace('store.objects.reference(event, "1").unwrap()', 'store.prepare(event, "1").unwrap().reference().clone()')
$source = '#![allow(dead_code)]' + "`n" + $source
$source | Set-Content -Encoding utf8 (Join-Path $probe 'src/lib.rs')
```

## 2. 向探针追加四个诊断

将以下 Rust 代码追加到上一步生成的 `src/lib.rs`（只在仓库外）。F1 不需要 SQL 破坏；F2 复用仓库原有的一次性沙箱凭据和真实子进程退出注入。B1 和 FIX-CORE-01 对照则明确破坏临时库索引，不能用它们证明合法发布会自行产生缺行。

```rust
#[test]
fn ledger_order_survives_normal_commit_and_reopen() {
    let root = TemporaryRoot::new().unwrap();
    let mut store = rehearsal(&root, "ledger-order");
    let cap = store.take_commit_capability().unwrap();
    let e = event(&mut store, 0);
    let (ticket, mut value, _pins) = proposal(&store, &e);
    let mut first = value.staged_ledger[0].clone();
    first.entry_id = "z-first".into();
    let mut second = first.clone();
    second.entry_id = "a-second".into();
    second.tendency_change.as_mut().unwrap().tendency_id = "another-tendency".into();
    value.staged_ledger = vec![first, second];
    let canonical = CoreTransactionV2::from_canonical(
        &value.canonical_bytes(config().encoding).unwrap(), config().encoding).unwrap();
    assert_eq!(canonical.staged_ledger[0].entry_id, "z-first");
    let staged = store.stage(ticket, value).unwrap();
    let result = store.commit(&cap, staged, normal()).unwrap();
    store.verify().unwrap();
    drop(store);
    let reopened = Store::<Rehearsal>::open_temporary(&root, "ledger-order", config()).unwrap();
    let reader = reopened.reader().unwrap();
    let history = reader.transactions_at(&result.commit_id, 10).unwrap();
    let expected: Vec<_> = history[0].staged_ledger.iter().map(|e| e.entry_id.clone()).collect();
    let actual: Vec<_> = reader.ledger_at(&result.commit_id, 10).unwrap().iter().map(|e| e.entry_id.clone()).collect();
    println!("canonical ledger order={expected:?}; ledger_at order={actual:?}; verify/reopen=OK");
    assert_eq!(actual, expected, "ledger query must retain canonical append order");
}

#[test]
fn boundary_missing_reference_row_allows_gc_to_remove_committed_payload() {
    let root = TemporaryRoot::new().unwrap();
    let mut store = rehearsal(&root, "missing-ref");
    let cap = store.take_commit_capability().unwrap();
    advance(&mut store, &cap, 0);
    let e = store.reader().unwrap().facts(1).unwrap().remove(0);
    let digest = core_types::scalar::hex(&e.payload_ref.digest.0);
    let path = store.path().to_path_buf();
    drop(store);
    // Deliberate logical index corruption, NOT a legal Store write or crash simulation.
    let db = rusqlite::Connection::open(path.join("store.sqlite")).unwrap();
    db.execute_batch("PRAGMA foreign_keys=ON; DROP TRIGGER immutable_visible_objects_DELETE; DROP TRIGGER immutable_object_refs_DELETE;").unwrap();
    db.execute("DELETE FROM visible_objects WHERE digest=?1", [&digest]).unwrap();
    db.execute("DELETE FROM object_refs WHERE digest=?1", [&digest]).unwrap();
    assert_eq!(db.query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0)).unwrap(), "ok");
    assert!(db.prepare("PRAGMA foreign_key_check").unwrap().query([]).unwrap().next().unwrap().is_none());
    drop(db);
    let mut store = Store::<Rehearsal>::open_temporary(&root, "missing-ref", config()).unwrap();
    store.verify().unwrap();
    let object_file = path.join("objects").join(format!("{digest}.obj"));
    assert!(object_file.exists());
    let removed = store.collect_garbage().unwrap();
    assert!(!object_file.exists());
    assert!(store.verify().is_err());
    println!("boundary: corrupted reference index passed open/verify; GC removed {removed} objects including committed payload; subsequent verify=ERROR");
}

#[test]
fn known_fix_core_01_missing_latest_group_resurrects_old_association() {
    let root = TemporaryRoot::new().unwrap();
    let mut store = rehearsal(&root, "known-evidence");
    let cap = store.take_commit_capability().unwrap();
    let mut commits = vec![];
    for n in 0..2 {
        let e = event(&mut store, n);
        let (ticket, mut value, _pins) = proposal(&store, &e);
        let mut group: EvidenceGroupV3 = fixture();
        group.group_id = "group".into();
        let mut allocation: EvidenceAllocationV3 = fixture();
        allocation.tendency_id = "tendency".into();
        group.allocations = if n == 0 { vec![allocation] } else { vec![] };
        let pin = store.prepare(&group, "1").unwrap();
        value.accepted_results.push(pin.reference().clone());
        let tx = store.stage(ticket, value).unwrap();
        commits.push(store.commit(&cap, tx, normal()).unwrap().commit_id);
    }
    assert!(store.reader().unwrap().evidence_for("tendency", &commits[1], 10).unwrap().is_empty());
    let path = store.path().to_path_buf();
    drop(store);
    let db = rusqlite::Connection::open(path.join("store.sqlite")).unwrap();
    db.execute_batch("DROP TRIGGER immutable_evidence_groups_DELETE;").unwrap();
    db.execute("DELETE FROM evidence_groups WHERE commit_id=?1", [&commits[1]]).unwrap();
    drop(db);
    let store = Store::<Rehearsal>::open_temporary(&root, "known-evidence", config()).unwrap();
    store.verify().unwrap();
    let reader = store.reader().unwrap();
    assert_eq!(reader.evidence_for("tendency", &commits[1], 10).unwrap().len(), 1);
    assert_eq!(reader.transactions_at(&commits[1], 10).unwrap().len(), 2);
    println!("known FIX-CORE-01: deleting only latest group index passed open/verify; removed association resurfaced; authoritative transaction count=2 unchanged");
}

#[test]
fn initialization_crash_cannot_admit_store_with_missing_reject_namespace() {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let root = TemporaryRoot::new().unwrap();
    let permit_name = "w02-child-permit-audit";
    let request = serde_json::json!({
        "protocol": "W02-owned-child-v1",
        "root": root.path().canonicalize().unwrap(),
        "permit": permit_name,
        "operation": "init",
        "point": "ReceiptKindSynced",
        "nth": 2,
        "tag": null
    });
    let bytes = serde_json::to_vec(&request).unwrap();
    let permit_path = root.path().join(permit_name);
    let mut permit = std::fs::OpenOptions::new().write(true).create_new(true).open(&permit_path).unwrap();
    permit.write_all(&bytes).unwrap();
    permit.sync_all().unwrap();
    drop(permit);
    let test_exe = std::env::var_os("NEMU_AUDIT_STORE_TEST_EXE").expect("set compiled core-store unit test executable");
    let mut child = Command::new(test_exe)
        .args(["--exact", "tests::crash_harness::crash_worker", "--nocapture"])
        .env("NEMU_W02_OWNED_CHILD", "1")
        .env_remove("NEMU_W02_CHILD_TEMP")
        .env_remove("NEMU_W02_CRASH")
        .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped())
        .spawn().unwrap();
    child.stdin.take().unwrap().write_all(&bytes).unwrap();
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(86), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(!permit_path.exists());
    assert!(String::from_utf8_lossy(&output.stdout).contains("W02_INJECT ReceiptKindSynced"));
    let reject_dir = root.path().join(".crash.w02-receipts-v1").join("reject");
    assert!(!reject_dir.exists());
    let mut store = Store::<Rehearsal>::open_temporary(&root, "crash", config()).unwrap();
    store.verify().unwrap();
    let cap = store.take_commit_capability().unwrap();
    advance(&mut store, &cap, 0);
    let audit = store.audit_ingress_rejection(IngressReason::AUTH_FAILED, None, &IngressAuditBudget { schema_version: 1, version: "audit-budget".into(), max_records: 3, max_bytes: 4096 });
    assert!(audit.is_err());
    assert_eq!(store.ingress_rejections().unwrap().len(), 1);
    store.verify().unwrap();
    drop(store);
    let reopened = Store::<Rehearsal>::open_temporary(&root, "crash", config());
    println!("init crash=86 after second receipt directory; first open/verify=OK; H commit=OK; audit={audit:?}; durable rejection rows=1; final open error={:?}", reopened.as_ref().err());
    assert!(reopened.is_ok(), "an admitted format-2 store must not become unopenable on its first rejection audit");
}
```

## 3. 串行运行并保留原始失败

先编译原有 core-store 单元测试以取得 F2 使用的真实崩溃 worker；此处 `--no-run` 不是原有测试全部通过的声明。整个探针会有 2 个失败、2 个诊断通过，退出码预期为 101。

```powershell
$env:CARGO_HOME = 'D:/Dev/cache/cargo'
$env:CARGO_TARGET_DIR = 'D:/Dev/cache/cargo-target'
$env:TMP = 'D:/Dev/cache/AUDIT-STORE-01-review'
$env:TEMP = $env:TMP
cargo +1.99.0 test --manifest-path core/Cargo.toml -p core-store --offline --locked --no-run --message-format=json 1> (Join-Path $probe 'build.jsonl')
if ($LASTEXITCODE -ne 0) { throw 'core-store compile failed' }
$build = Get-Content (Join-Path $probe 'build.jsonl') | ForEach-Object { $_ | ConvertFrom-Json }
$env:NEMU_AUDIT_STORE_TEST_EXE = ($build | Where-Object { $_.reason -eq 'compiler-artifact' -and $_.target.name -eq 'core_store' -and $_.profile.test -and $_.executable } | Select-Object -Last 1).executable
if (-not $env:NEMU_AUDIT_STORE_TEST_EXE) { throw 'missing core-store test executable' }
cargo +1.99.0 generate-lockfile --offline --manifest-path (Join-Path $probe 'Cargo.toml')
if ($LASTEXITCODE -ne 0) { throw 'probe resolution failed' }
cargo +1.99.0 test --offline --locked --manifest-path (Join-Path $probe 'Cargo.toml') -- --test-threads=1 --nocapture 2>&1 | Tee-Object (Join-Path $probe 'diagnostics.log')
# 不把预期的产品反例失败改成通过；查看 exit 101 和逐项断言。
```

离线命令要求该机器已缓存仓库锁定的 Rust/dependencies；缺依赖时明确报告环境未满足，不把它计为已复现。

本次实际探针位置为 `D:/Dev/cache/AUDIT-STORE-01/probe/`；实际运行记录和结果见 [报告](REPORT.md)。合成夹具用于存储层检查，不运行 G/C 数学计算、不产生行为验收结论。
