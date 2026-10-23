use super::*;
use std::{path::PathBuf, time::Instant};

/// Explicit real-model acceptance. Unit/CI defaults never download a model or
/// open any user store. Run with the documented D: model directory and --ignored.
#[test]
#[ignore = "explicit pinned local CPU model required; see fact-view README"]
fn real_cpu_multilingual_restart_recall_and_100_retractions() {
    let directory = PathBuf::from(std::env::var_os("NEMU_W19_MODEL").expect("explicit model path"));
    let start = Instant::now();
    let mut model = CpuEmbedder::load(&directory).unwrap();
    let load_ms = start.elapsed().as_secs_f64() * 1000.0;
    let mut w = World::new();
    let withdrawn = w.text("合成角色阿岚最爱喝茉莉花茶。");
    w.text("The synthetic character Mina keeps a red bicycle in the garage.");
    w.text("架空の人物ソラは毎週土曜日に図書館へ行きます。");
    for text in [
        "合成登山队计划六月攀登雪山。",
        "A synthetic bakery closes at five in the evening.",
        "架空の猫モモの毛色は白です。",
        "合成项目的截止日期是九月十五日。",
        "Synthetic astronomers observed a distant galaxy.",
        "架空の島では明日は雨が降ります。",
    ] {
        w.text(text);
    }
    // Restart the actual temporary core-store, not just the derived index.
    let mut w = w.reopen();
    let cases = [
        ("阿岚喜欢喝什么饮料？", "event-0"),
        ("What does Aran like to drink?", "event-0"),
        ("アランの好きな飲み物は？", "event-0"),
        ("Mina 的红色自行车停在哪里？", "event-1"),
        ("Where is Mina's bicycle stored?", "event-1"),
        ("Mina の赤い自転車はどこ？", "event-1"),
        ("Sora 每周六去哪里？", "event-2"),
        ("Where does Sora go on Saturdays?", "event-2"),
        ("ソラは土曜日にどこへ行く？", "event-2"),
    ];
    let reader = w.store.reader().unwrap();
    let prefix = w.prefix();
    let start = Instant::now();
    let view = FactView::rebuild(&reader, &prefix, 100, Some(&mut model)).unwrap();
    let rebuild_ms = start.elapsed().as_secs_f64() * 1000.0;
    let mut rows = Vec::new();
    let mut latencies = Vec::new();
    for (text, expected) in cases {
        let q = query(&prefix, Purpose::Task, text);
        let start = Instant::now();
        let result = view.execute(&q, &access(), Some(&mut model)).unwrap();
        let ms = start.elapsed().as_secs_f64() * 1000.0;
        latencies.push(ms);
        assert_eq!(result.projection.status, QueryStatus::OK);
        let rank = result
            .projection
            .items
            .iter()
            .position(|i| i.source_id == expected)
            .map(|n| n + 1);
        let vector_rank = result.trace.channels[3]
            .iter()
            .position(|id| id == expected)
            .map(|n| n + 1);
        rows.push(serde_json::json!({"query":text,"expected":expected,"rank":rank,"vector_rank":vector_rank,"ms":ms,"steps":result.trace.steps_used}));
        assert!(
            result
                .trace
                .query_vector
                .as_ref()
                .is_some_and(|v| v.len() == 384)
        );
        assert!(
            result
                .trace
                .audit_record(&result.projection)
                .unwrap()
                .text
                .contains("query_vector_bits")
        );
    }
    let hits = rows
        .iter()
        .filter(|r| r["rank"].as_u64().is_some_and(|n| n <= 3))
        .count();
    drop(view);
    drop(reader);
    w.add(
        &UserRetraction {
            schema_version: 1,
            target_refs: vec![withdrawn.id],
            note: None,
        },
        "user",
        &[],
        None,
    );
    let reader = w.store.reader().unwrap();
    let prefix = w.prefix();
    let view = FactView::rebuild(&reader, &prefix, 100, Some(&mut model)).unwrap();
    let mut leaks = 0;
    for n in 0..100 {
        let result = view
            .execute(
                &query(&prefix, Purpose::Task, cases[n % 3].0),
                &access(),
                Some(&mut model),
            )
            .unwrap();
        leaks += result
            .projection
            .items
            .iter()
            .filter(|i| i.source_id == "event-0")
            .count();
        assert!(
            !result.projection.items.is_empty(),
            "positive controls remain retrievable"
        );
        assert!(
            !result
                .trace
                .channels
                .iter()
                .flatten()
                .any(|id| id == "event-0")
        );
    }
    latencies.sort_by(f64::total_cmp);
    let report = serde_json::json!({"version":"w19-eval-skeleton-v1","status":"passed","synthetic_facts":9,"languages":["zh","en","ja"],"model":model.version(),"load_ms":load_ms,"rebuild_ms":rebuild_ms,"recall_at_3":{"hits":hits,"queries":rows.len()},"query_p50_ms":latencies[latencies.len()/2],"query_max_ms":latencies.last(),"withdrawal_queries":100,"withdrawal_leaks":leaks,"cases":rows,"limits":"Small synthetic restart skeleton, not the P6-13 multi-year/full-answer 90% acceptance. Latencies are observational on this host, not an exclusive CPU benchmark."});
    println!("W19_EVAL_JSON={report}");
    assert_eq!(leaks, 0);
    assert_eq!(
        hits,
        cases.len(),
        "small multilingual positive-control recall@3"
    );
    // An altered model file in an owned temporary directory fails before ORT.
    let corrupt = tempfile::tempdir().unwrap();
    std::fs::write(corrupt.path().join("model.onnx"), b"corrupt").unwrap();
    assert!(CpuEmbedder::load(corrupt.path()).is_err());
}
