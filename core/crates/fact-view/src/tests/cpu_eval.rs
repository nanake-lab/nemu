use super::*;
use std::{path::PathBuf, time::Instant};

#[test]
#[ignore = "explicit diagnostic; immutable descriptor only, no stores"]
fn profile_builtin_codec_descriptor_construction() {
    use core_types::contract::Schema;
    let value = UserText {
        schema_version: 1,
        text: "synthetic descriptor profile".into(),
    };
    let json = serde_json::to_value(&value).unwrap();
    let schema = Schema::builtin().unwrap();
    let expected = schema
        .encode_canonical(UserText::NAME, &json, crate::query::LIMITS)
        .unwrap();
    let start = Instant::now();
    for _ in 0..100 {
        assert_eq!(
            Schema::builtin()
                .unwrap()
                .encode_canonical(UserText::NAME, &json, crate::query::LIMITS)
                .unwrap(),
            expected
        );
    }
    let fresh_ms = start.elapsed().as_secs_f64() * 1000.0;
    let start = Instant::now();
    for _ in 0..100 {
        assert_eq!(
            schema
                .encode_canonical(UserText::NAME, &json, crate::query::LIMITS)
                .unwrap(),
            expected
        );
    }
    let reused_ms = start.elapsed().as_secs_f64() * 1000.0;
    let start = Instant::now();
    for _ in 0..100 {
        assert_eq!(
            value.canonical_bytes(crate::query::LIMITS).unwrap(),
            expected
        );
    }
    let wire_codec_ms = start.elapsed().as_secs_f64() * 1000.0;
    println!(
        "W19_DESCRIPTOR_PROFILE={{\"rounds\":100,\"fresh_descriptor_ms\":{fresh_ms},\"reused_descriptor_ms\":{reused_ms},\"wire_codec_ms\":{wire_codec_ms}}}"
    );
}

struct TimedModel {
    inner: CpuEmbedder,
    query_ms: f64,
}
impl Embedder for TimedModel {
    fn version(&self) -> &str {
        self.inner.version()
    }
    fn passages(&mut self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        self.inner.passages(texts)
    }
    fn query(&mut self, text: &str) -> Result<Vec<f32>> {
        let start = Instant::now();
        let result = self.inner.query(text);
        self.query_ms = start.elapsed().as_secs_f64() * 1000.0;
        result
    }
}

/// Explicit real-model acceptance. Unit/CI defaults never download a model or
/// open any user store. Run with the documented D: model directory and --ignored.
#[test]
#[ignore = "explicit pinned local CPU model required; see fact-view README"]
fn real_cpu_multilingual_restart_recall_and_100_retractions() {
    let directory = PathBuf::from(std::env::var_os("NEMU_W19_MODEL").expect("explicit model path"));
    let start = Instant::now();
    let model = CpuEmbedder::load(&directory).unwrap();
    let mut model = TimedModel {
        inner: model,
        query_ms: 0.0,
    };
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
    let mut core_latencies = Vec::new();
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
        assert_eq!(
            vector_rank,
            Some(1),
            "query={text}, expected={expected}; a constant embedding/retriever must fail the multilingual positive controls"
        );
        assert_eq!(
            rank,
            Some(1),
            "RRF must preserve these distinct positive controls"
        );
        let task_embedding_ms = model.query_ms;
        // ADR 0012's first-round latency concerns c_core, which still excludes
        // reactions. Keep the original, broader c_task measurements too.
        let start = Instant::now();
        let core = view
            .execute(
                &query(&prefix, Purpose::Core, text),
                &access(),
                Some(&mut model),
            )
            .unwrap();
        let core_ms = start.elapsed().as_secs_f64() * 1000.0;
        core_latencies.push(core_ms);
        assert_eq!(core.projection.status, QueryStatus::OK);
        assert_eq!(
            core.projection.items.first().map(|i| i.source_id.as_str()),
            Some(expected)
        );
        assert!(core.memories().iter().all(|m| m.marker.is_none()));
        rows.push(serde_json::json!({"query":text,"expected":expected,"rank":rank,"vector_rank":vector_rank,"ms":ms,"embedding_ms":task_embedding_ms,"steps":result.trace.steps_used,"core_first_round_ms":core_ms}));
        assert!(
            result
                .trace
                .query_vector
                .as_ref()
                .is_some_and(|v| v.len() == 384)
        );
        let norm = result
            .trace
            .query_vector
            .as_ref()
            .unwrap()
            .iter()
            .map(|v| (*v as f64).powi(2))
            .sum::<f64>();
        assert!((norm - 1.0).abs() < 0.0001, "E5 normalization");
        let audit = result.trace.audit_record(&result.projection).unwrap();
        let payload = String::from_utf8(audit.payload.0).unwrap();
        assert!(payload.contains("query_vector_bits"));
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
    core_latencies.sort_by(f64::total_cmp);
    let report = serde_json::json!({"version":"w19-eval-skeleton-v2","status":"passed","synthetic_facts":9,"languages":["zh","en","ja"],"model":model.version(),"ort_build":ort::info(),"load_ms":load_ms,"rebuild_ms":rebuild_ms,"recall_at_3":{"hits":hits,"queries":rows.len()},"query_p50_ms":latencies[latencies.len()/2],"query_max_ms":latencies.last(),"core_first_round_p50_ms":core_latencies[core_latencies.len()/2],"core_first_round_max_ms":core_latencies.last(),"withdrawal_queries":100,"withdrawal_leaks":leaks,"cases":rows,"limits":"Small synthetic restart skeleton, not the P6-13 multi-year/full-answer 90% acceptance. Latencies are observational on this host, not an exclusive CPU benchmark. Full c_task pages include marked historical reactions; c_core first-round timings are separate."});
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
