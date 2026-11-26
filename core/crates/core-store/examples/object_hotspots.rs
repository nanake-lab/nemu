//! Run with `cargo run --release --locked -p core-store --example object_hotspots`.
//! Uses the same public Store entry points on the baseline and the candidate.
use core_store::{Experiment, Result, Store, StoreConfig, TemporaryRoot};
use core_types::{
    contract::Limits,
    generated::{Lineage, LineageHeaderV2, RSnapshotV3, UserText},
    scalar::{DecimalI64, DecimalU64},
};
use std::time::{Duration, Instant};

fn main() -> Result<()> {
    // Same resource bounds as the core-store synthetic object fixtures.
    let config = StoreConfig {
        encoding: Limits {
            max_bytes: 4 * 1024 * 1024,
            max_items: 4096,
            max_depth: 64,
        },
        busy_timeout: Duration::from_millis(30),
        max_pending_records: 32,
        max_pending_bytes: 1024 * 1024,
        max_object_bytes: 4 * 1024 * 1024,
        max_store_bytes: 64 * 1024 * 1024,
        max_sql_pages: 8192,
    };
    let root = TemporaryRoot::new()?;
    let header = LineageHeaderV2 {
        schema_version: 1,
        lineage: Lineage::EXPERIMENT,
        created_at_ns: DecimalI64(1),
        genesis_manifest_digest: None,
        world_id: Some("object-probe".into()),
        world_seed: Some(DecimalU64(1)),
    };
    let store = Store::<Experiment>::create_temporary(&root, "probe", header, "probe", config)?;
    let value = UserText {
        schema_version: 1,
        text: "synthetic object".into(),
    };
    // A plain leaf and an NSI1 index referring to that leaf exercise reference
    // creation, both read_depth paths, and traversal of a nonempty closure.
    let started = Instant::now();
    let leaf = store.prepare(&value, "1")?;
    let leaf_new = started.elapsed();
    let snapshot = RSnapshotV3 {
        schema_version: 1,
        revision: "probe".into(),
        law_version: leaf.reference().clone(),
        tendencies: vec![],
    };
    let started = Instant::now();
    let indexed = store.prepare_r(&snapshot)?;
    let index_new = started.elapsed();

    println!("operation,sample,elapsed_ns");
    println!("leaf_new,1,{}", leaf_new.as_nanos());
    println!("index_new,1,{}", index_new.as_nanos());
    // Keep every repetition so the caller can inspect filesystem variation.
    for sample in 1..=10 {
        let started = Instant::now();
        let reused_leaf = store.prepare(&value, "1")?;
        let leaf_time = started.elapsed();
        let started = Instant::now();
        let reused_index = store.prepare_r(&snapshot)?;
        let index_time = started.elapsed();
        assert_eq!(reused_leaf.reference(), leaf.reference());
        assert_eq!(reused_index.reference(), indexed.reference());
        println!("leaf_reuse,{sample},{}", leaf_time.as_nanos());
        println!("index_reuse,{sample},{}", index_time.as_nanos());
    }
    Ok(())
}
