#![forbid(unsafe_code)]
use r_kappa_01::{kernel, scenario};
use serde_json::json;
use std::{
    env,
    fs::{self, OpenOptions},
    io::{BufWriter, Write},
    path::Path,
    time::Instant,
};

fn save(path: &Path, value: &impl serde::Serialize) {
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .unwrap();
    serde_json::to_writer_pretty(BufWriter::new(file), value).unwrap();
}

fn main() {
    let args: Vec<_> = env::args().collect();
    assert_eq!(args.len(), 2, "usage: r-kappa-01 <NEW output directory>");
    let output = Path::new(&args[1]);
    fs::create_dir(output).expect("evidence directory must not already exist");
    let start = Instant::now();
    let fixtures = scenario::fixtures();
    save(&output.join("fixtures.json"), &fixtures);
    save(
        &output.join("parameters.json"),
        &json!({"envelope":scenario::envelope(),
            "initial_state":kernel::vector(&kernel::initial()),"b":[0.0,0.0,0.0,0.0],"g":[1.0,1.0,1.0,1.0],"lambda":[1.0,1.0,1.0,1.0],
        "r_plus":{"pi":0.75,"r":0.1,"direction":[0.0,0.0,0.0,1.0]},
        "assertion":"synthetic pre-recorded A/M; no language model or growth implementation"}),
    );
    let mut summaries = Vec::new();
    for fixture in &fixtures {
        for group in ["A", "B", "C"] {
            let (traces, summary) = scenario::run(fixture, group);
            let file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(output.join(format!("{}-{group}.jsonl", fixture.scenario)))
                .unwrap();
            let mut writer = BufWriter::new(file);
            for trace in traces {
                serde_json::to_writer(&mut writer, &trace).unwrap();
                writeln!(writer).unwrap();
            }
            writer.flush().unwrap();
            summaries.push(summary);
        }
    }
    save(&output.join("summary.json"), &summaries);
    save(
        &output.join("resource.json"),
        &json!({"wall_ms":start.elapsed().as_secs_f64()*1000.0,
        "trajectories":9,"events":9*512,"threads":1,"model_calls":0,"gpu_calls":0,
        "simulation_seconds_per_trajectory":256.2}),
    );
    println!(
        "{}",
        json!({"output":output,"trajectories":9,"events":4608})
    );
}
