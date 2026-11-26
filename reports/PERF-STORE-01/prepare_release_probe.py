"""Install the synthetic #240 release probe in a disposable measurement checkout."""

from pathlib import Path
import sys

SCALE_TEST = r'''
#[tokio::test]
#[ignore = "temporary review measurement on synthetic histories"]
async fn review_scale_probe() {
    let root = TemporaryRoot::new().unwrap();
    let (mut g, _listener, _, _) = setup(&root, "scale-review", deployment());
    let fixture_reader = g.store.reader().unwrap();
    let payload = text();
    let reference = payload.reference(limits()).unwrap();
    let uploaded = crate::uploads::Uploads::default();
    let mut slot = None;

    for n in 0..128 {
        if [1, 7, 31, 127].contains(&n) {
            core_types::profile_metrics::reset();
            let started = std::time::Instant::now();
            g.store.receipt_reader(&mut slot).unwrap();
            eprintln!(
                "SYS01_SCALE profile={} operation=verified_prefix_before_next_input commits={} elapsed_ns={}",
                std::env::var("SYS01_PROFILE").unwrap(),
                n,
                started.elapsed().as_nanos()
            );
        }

        let (e, source) = event(&payload, n);
        let validated = validate_event(
            &session(),
            Grants::default(),
            e,
            payload.clone(),
            source,
            limits(),
        )
        .unwrap();
        ingest_validated(&mut g.store, validated).unwrap();
        super::runtime_fixture::commit_result_with_reader(
            &mut g.store,
            &g.handler.cap,
            &fixture_reader,
        );

        let commits = n + 1;
        if [2, 8, 32, 128].contains(&commits) {
            core_types::profile_metrics::reset();
            let started = std::time::Instant::now();
            g.store.receipt_reader(&mut slot).unwrap();
            let elapsed = started.elapsed();
            eprintln!(
                "SYS01_SCALE profile={} commits={} operation=new_head elapsed_ns={}",
                std::env::var("SYS01_PROFILE").unwrap(),
                commits,
                elapsed.as_nanos()
            );

            core_types::profile_metrics::reset();
            let started = std::time::Instant::now();
            let value: UserText = uploaded.read(&g.store, &reference, limits()).unwrap();
            let elapsed = started.elapsed();
            assert_eq!(value.text, "synthetic input");
            eprintln!(
                "SYS01_SCALE profile={} commits={} operation=upload_fallback elapsed_ns={}",
                std::env::var("SYS01_PROFILE").unwrap(),
                commits,
                elapsed.as_nanos()
            );

            if std::env::var("SYS01_PROFILE").unwrap() == "release" {
                core_types::profile_metrics::reset();
                let started = std::time::Instant::now();
                let check = g.store.full_publication_check().unwrap();
                let mut reader = check().unwrap();
                let elapsed = started.elapsed();
                let metrics = core_types::profile_metrics::snapshot();
                let accounted = metrics.integrity_ns
                    + metrics.object_io_ns
                    + metrics.canonical_decode_ns
                    + metrics.witness_ns
                    + metrics.schema_json_ns;
                let other = elapsed.as_nanos().saturating_sub(accounted);
                eprintln!(
                    "SYS01_FULL commits={} total_ns={} integrity_ns={} object_io_ns={} canonical_decode_ns={} witness_ns={} other_ns={} decode_calls={} schema_json_ns={} schema_calls={}",
                    commits,
                    elapsed.as_nanos(),
                    metrics.integrity_ns,
                    metrics.object_io_ns,
                    metrics.canonical_decode_ns,
                    metrics.witness_ns,
                    other,
                    metrics.decode_calls,
                    metrics.schema_json_ns,
                    metrics.schema_calls
                );
                g.store.accept_publication(&mut reader).unwrap();
            }
        }
    }
}
'''

METRICS_RS = r'''
use std::{
    cell::{Cell, RefCell},
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Debug, Default)]
pub struct Snapshot {
    pub integrity_ns: u128,
    pub object_io_ns: u128,
    pub canonical_decode_ns: u128,
    pub witness_ns: u128,
    pub decode_calls: u64,
    pub schema_json_ns: u128,
    pub schema_calls: u64,
}

thread_local! {
    static VALUES: RefCell<Snapshot> = RefCell::new(Snapshot::default());
    static DECODE_DEPTH: Cell<usize> = const { Cell::new(0) };
    static WITNESS_DEPTH: Cell<usize> = const { Cell::new(0) };
}

pub fn reset() {
    VALUES.with(|values| *values.borrow_mut() = Snapshot::default());
}

pub fn snapshot() -> Snapshot {
    VALUES.with(|values| *values.borrow())
}

fn in_witness() -> bool {
    WITNESS_DEPTH.with(|depth| depth.get() != 0)
}

pub fn integrity(elapsed: Duration) {
    if !in_witness() {
        VALUES.with(|values| values.borrow_mut().integrity_ns += elapsed.as_nanos());
    }
}

pub fn object_io(elapsed: Duration) {
    if !in_witness() {
        VALUES.with(|values| values.borrow_mut().object_io_ns += elapsed.as_nanos());
    }
}

pub struct SchemaJsonScope {
    started: Option<Instant>,
}

pub fn schema_json_scope() -> SchemaJsonScope {
    let eligible = !in_witness() && DECODE_DEPTH.with(|depth| depth.get() == 0);
    SchemaJsonScope {
        started: eligible.then(Instant::now),
    }
}

impl Drop for SchemaJsonScope {
    fn drop(&mut self) {
        if let Some(started) = self.started {
            VALUES.with(|values| {
                let mut values = values.borrow_mut();
                values.schema_json_ns += started.elapsed().as_nanos();
                values.schema_calls += 1;
            });
        }
    }
}

pub struct DecoderScope {
    started: Instant,
    outermost: bool,
}

pub fn decoder_scope() -> DecoderScope {
    let outermost = DECODE_DEPTH.with(|depth| {
        let old = depth.get();
        depth.set(old + 1);
        old == 0
    });
    DecoderScope {
        started: Instant::now(),
        outermost,
    }
}

impl Drop for DecoderScope {
    fn drop(&mut self) {
        DECODE_DEPTH.with(|depth| depth.set(depth.get() - 1));
        if self.outermost && !in_witness() {
            VALUES.with(|values| {
                let mut values = values.borrow_mut();
                values.canonical_decode_ns += self.started.elapsed().as_nanos();
                values.decode_calls += 1;
            });
        }
    }
}

pub struct WitnessScope {
    started: Instant,
    outermost: bool,
}

pub fn witness_scope() -> WitnessScope {
    let outermost = WITNESS_DEPTH.with(|depth| {
        let old = depth.get();
        depth.set(old + 1);
        old == 0
    });
    WitnessScope {
        started: Instant::now(),
        outermost,
    }
}

impl Drop for WitnessScope {
    fn drop(&mut self) {
        WITNESS_DEPTH.with(|depth| depth.set(depth.get() - 1));
        if self.outermost {
            VALUES.with(|values| values.borrow_mut().witness_ns += self.started.elapsed().as_nanos());
        }
    }
}
'''

def replace_once(path: Path, old: str, new: str) -> None:
    source = path.read_text(encoding="utf-8")
    count = source.count(old)
    if count != 1:
        raise RuntimeError(f"expected one match in {path}, found {count}: {old[:100]!r}")
    path.write_text(source.replace(old, new), encoding="utf-8", newline="\n")

def inject(workspace: Path) -> None:
    types = workspace / "crates/core-types/src"
    store = workspace / "crates/core-store/src"

    (types / "profile_metrics.rs").write_text(METRICS_RS.lstrip(), encoding="utf-8", newline="\n")
    lib = types / "lib.rs"
    lib.write_text(lib.read_text(encoding="utf-8") + "\npub mod profile_metrics;\n", encoding="utf-8", newline="\n")

    replace_once(
        types / "canonical.rs",
        "    pub fn decode_canonical(&self, name: &str, b: &[u8], l: Limits, depth: usize) -> Result<Value> {\n",
        "    pub fn decode_canonical(&self, name: &str, b: &[u8], l: Limits, depth: usize) -> Result<Value> {\n"
        "        let _profile_decoder = crate::profile_metrics::decoder_scope();\n",
    )

    objects = store / "objects.rs"
    replace_once(
        objects,
        "        let path = self.path(r)?;\n        platform::no_links(&path)?;\n",
        "        let path = self.path(r)?;\n        let object_io_started = std::time::Instant::now();\n"
        "        platform::no_links(&path)?;\n",
    )
    replace_once(
        objects,
        "        let disk = fs::read(path)?;\n",
        "        let disk = fs::read(path)?;\n"
        "        core_types::profile_metrics::object_io(object_io_started.elapsed());\n",
    )
    replace_once(
        objects,
        "            let disk = fs::read(self.path(&r)?)?;\n",
        "            let object_path = self.path(&r)?;\n"
        "            let object_io_started = std::time::Instant::now();\n"
        "            let disk = fs::read(&object_path)?;\n"
        "            core_types::profile_metrics::object_io(object_io_started.elapsed());\n",
    )

    contract = types / "contract.rs"
    replace_once(
        contract,
        "    pub fn builtin() -> Result<Self> {\n",
        "    pub fn builtin() -> Result<Self> {\n"
        "        let _profile_schema_json = crate::profile_metrics::schema_json_scope();\n",
    )

    recovery = store / "recovery.rs"
    replace_once(
        recovery,
        '    require(\n'
        '        db.query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0))? == "ok",\n'
        '        "database integrity failure",\n'
        '    )?;\n',
        '    let integrity_started = std::time::Instant::now();\n'
        '    let integrity = db.query_row("PRAGMA integrity_check", [], |r| r.get::<_, String>(0));\n'
        '    core_types::profile_metrics::integrity(integrity_started.elapsed());\n'
        '    require(integrity? == "ok", "database integrity failure")?;\n',
    )

    fixture = workspace / "crates/gateway/src/tests/runtime_fixture.rs"
    replace_once(
        fixture,
        "pub(super) fn commit_result(\n"
        "    store: &mut Store<Rehearsal>,\n"
        "    cap: &CommitCapability<Rehearsal>,\n"
        ") -> CoreCommitResultV2 {\n"
        "    let event = store.next_raw().unwrap().unwrap();\n",
        "pub(super) fn commit_result(\n"
        "    store: &mut Store<Rehearsal>,\n"
        "    cap: &CommitCapability<Rehearsal>,\n"
        ") -> CoreCommitResultV2 {\n"
        "    let reader = store.reader().unwrap();\n"
        "    commit_result_with_reader(store, cap, &reader)\n"
        "}\n\n"
        "pub(super) fn commit_result_with_reader(\n"
        "    store: &mut Store<Rehearsal>,\n"
        "    cap: &CommitCapability<Rehearsal>,\n"
        "    reader: &core_store::Reader<Rehearsal>,\n"
        ") -> CoreCommitResultV2 {\n"
        "    let event = store.next_raw().unwrap().unwrap();\n",
    )
    replace_once(
        fixture,
        "    if let Some(old) = store.current().unwrap() {\n"
        "        let reader = store.reader().unwrap();\n",
        "    if let Some(old) = store.current().unwrap() {\n",
    )

    reader = store / "reader.rs"
    replace_once(
        reader,
        "        crate::witness::admit(&db, &root, &objects, false)?;\n",
        "        {\n"
        "            let _witness_scope = core_types::profile_metrics::witness_scope();\n"
        "            crate::witness::admit(&db, &root, &objects, false)?;\n"
        "        }\n",
    )
    replace_once(
        reader,
        "        let receipts = if prefix.is_none() {\n"
        "            Some(crate::witness::capture(&root, &objects)?)\n"
        "        } else {\n",
        "        let receipts = if prefix.is_none() {\n"
        "            let _witness_scope = core_types::profile_metrics::witness_scope();\n"
        "            Some(crate::witness::capture(&root, &objects)?)\n"
        "        } else {\n",
    )
    replace_once(
        reader,
        "                    crate::recovery::check_new_witnesses(\n",
        "                    let _witness_scope = core_types::profile_metrics::witness_scope();\n"
        "                    crate::recovery::check_new_witnesses(\n",
    )
    replace_once(
        reader,
        "            crate::witness::check(&db, &objects, &receipts.unwrap())?;\n",
        "            let _witness_scope = core_types::profile_metrics::witness_scope();\n"
        "            crate::witness::check(&db, &objects, &receipts.unwrap())?;\n",
    )

    service = workspace / "crates/gateway/src/tests/service.rs"
    service.write_text(
        service.read_text(encoding="utf-8") + SCALE_TEST,
        encoding="utf-8",
        newline="\n",
    )


if __name__ == "__main__":
    inject(Path(sys.argv[1]))
