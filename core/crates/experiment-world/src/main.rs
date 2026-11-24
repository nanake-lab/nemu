use core_kernel::recompute::Kernel;
use experiment_world::{adapter::*, probes, resources::*, world::*};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, BufWriter, Write},
    path::Path,
    time::Instant,
};
fn create(p: &Path) -> Result<BufWriter<File>> {
    Ok(BufWriter::new(
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(p)
            .map_err(err)?,
    ))
}
fn write_json(p: &Path, v: &impl serde::Serialize) -> Result<()> {
    let mut f = create(p)?;
    serde_json::to_writer(&mut f, v).map_err(err)?;
    f.flush().map_err(err)
}
fn line(f: &mut impl Write, v: &impl serde::Serialize) -> Result<()> {
    serde_json::to_writer(&mut *f, v).map_err(err)?;
    writeln!(f).map_err(err)
}
fn prepare(root: &Path) -> Result<()> {
    fs::create_dir(root).map_err(err)?;
    write_json(&root.join("runs.json"), &runs())?;
    write_json(&root.join("probes.json"), &probes())?;
    write_json(&root.join("candidates.json"),&candidates().iter().map(|c|json!({"id":c.record_id,"concept":c.concept,"predicate":c.predicate,"direction":c.direction,"relation":c.relation,"proposal_event_ordinal":1})).collect::<Vec<_>>())?;
    write_json(&root.join("match-table.json"),&["U","V"].into_iter().flat_map(|p|contexts().into_iter().map(move|c|json!({"person":p,"context":c,"mu_one":ids(p,c),"all_other_mu":0,"observed":true,"exogenous":true,"independent_cause":"ordinal"}))).collect::<Vec<_>>())?;
    let mut resource = create(&root.join("law-resource.json"))?;
    resource.write_all(&binding_bytes()).map_err(err)?;
    resource.flush().map_err(err)?;
    let start = genesis()?;
    let theta = core_kernel::dynamics::export_snapshot(
        &start.theta,
        "genesis",
        &reference(),
        &fuses(),
        LIMITS,
    )
    .map_err(err)?;
    write_json(
        &root.join("parameters.json"),
        &json!({"lineage_kind":"EXPERIMENT","version":VERSION,"G":parameters(),"C":theta.parameters,"F":{"g_max":drive_law().g_max,"r_max":drive_law().r_max,"w_min":drive_law().w_min,"kappa":drive_law().kappa},"genesis":{"R":[],"x":[0,0,0,0],"theta":theta},"capacity":100,"protected":1,"attachment_prior":0,"source":"core-kernel/src/recompute/tests/{fixture,legacy}.rs; kappa profiles/structural_fixture/fast/fixture.json; no calibrated life parameters"}),
    )?;
    for run in runs().iter().filter(|r| r.status == "ready") {
        let mut f = create(&root.join(format!("{}.jsonl", run.id)))?;
        for e in events(run) {
            line(&mut f, &e)?;
        }
        f.flush().map_err(err)?;
    }
    let mut hashes = std::collections::BTreeMap::new();
    for entry in fs::read_dir(root).map_err(err)? {
        let p = entry.map_err(err)?.path();
        hashes.insert(
            p.file_name().unwrap().to_string_lossy().into_owned(),
            hash(&fs::read(&p).map_err(err)?),
        );
    }
    write_json(&root.join("sha256.json"), &hashes)
}
fn execute(package: &Path, id: &str, out: &Path) -> Result<()> {
    if cfg!(debug_assertions) {
        return Err("long runs require release build".into());
    }
    let manifest: std::collections::BTreeMap<String, String> =
        serde_json::from_slice(&fs::read(package.join("sha256.json")).map_err(err)?)
            .map_err(err)?;
    for (name, expected_hash) in &manifest {
        if hash(&fs::read(package.join(name)).map_err(err)?) != *expected_hash {
            return Err(format!("frozen input mismatch: {name}"));
        }
    }
    let binding: serde_json::Value =
        serde_json::from_slice(&fs::read(package.join("parameters.json")).map_err(err)?)
            .map_err(err)?;
    if binding["lineage_kind"] != "EXPERIMENT"
        || binding["version"] != VERSION
        || fs::read(package.join("law-resource.json")).map_err(err)? != binding_bytes()
    {
        return Err("not this synthetic execution package".into());
    }
    let runs: Vec<Run> =
        serde_json::from_slice(&fs::read(package.join("runs.json")).map_err(err)?).map_err(err)?;
    let run = runs
        .iter()
        .find(|r| r.id == id && r.status == "ready")
        .ok_or("unknown run")?;
    let ps: Vec<Probe> =
        serde_json::from_slice(&fs::read(package.join("probes.json")).map_err(err)?)
            .map_err(err)?;
    let input = BufReader::new(File::open(package.join(format!("{id}.jsonl"))).map_err(err)?)
        .lines()
        .map(|l| serde_json::from_str::<Event>(&l.map_err(err)?).map_err(err))
        .collect::<Result<Vec<_>>>()?;
    fs::create_dir(out).map_err(err)?;
    let mut trace = experiment_world::compress::Compressed::create(&out.join("events.jsonl.gz"))
        .map_err(err)?;
    let mut days =
        experiment_world::compress::Compressed::create(&out.join("daily.jsonl.gz")).map_err(err)?;
    let mut s = genesis()?;
    let mut cursor = 0;
    let mut actuals = Vec::new();
    let timer = Instant::now();
    let mut slow = 0.;
    let mut status = "passed";
    let mut failure = None;
    line(&mut days, &probes::daily(&s, 0, &ps)?)?;
    'life: for day in 1..=run.days {
        while cursor < input.len() && input[cursor].day == day {
            let mut actual = input[cursor].clone();
            if run.world.starts_with("O5-") {
                let fields: Vec<_> = run.world.split('-').collect();
                let mut c = if fields[1] == "positive" {
                    0.125
                } else {
                    -0.125
                };
                if day >= 181
                    && (fields[3] == "permanent" || (fields[3] == "temporary" && day <= 210))
                {
                    c = -0.25;
                }
                let k = match fields[2] {
                    "k0" => 0.,
                    "k025" => 0.25,
                    "k050" => 0.5,
                    _ => return Err("feedback rule".into()),
                };
                actual.a[0] = c + k * s.x.nature[0];
            }
            let e = &actual;
            let t = Instant::now();
            match advance(&s, e) {
                Ok(next) => {
                    let group_delta = {
                        let old: std::collections::BTreeMap<_, _> =
                            s.groups.iter().map(|g| (&g.group_id, g)).collect();
                        next.groups
                            .iter()
                            .filter(|g| old.get(&g.group_id).is_none_or(|before| *before != *g))
                            .cloned()
                            .collect::<Vec<_>>()
                    };
                    s = next;
                    cursor += 1;
                    actuals.push(actual.clone());
                    line(
                        &mut trace,
                        &json!({"event":e,"state":probes::observed(&s),"r_full":s.r,"group_delta":group_delta,"theta":core_kernel::dynamics::export_snapshot(&s.theta,"event",&reference(),&fuses(),LIMITS).map_err(err)?}),
                    )?;
                }
                Err(reason) => {
                    status = "failed";
                    failure = Some(
                        json!({"ordinal":e.ordinal,"reason":reason,"input":e,"committed_events":cursor}),
                    );
                    break 'life;
                }
            }
            slow += t.elapsed().as_secs_f64();
        }
        match probes::daily(&s, day, &ps) {
            Ok(v) => line(&mut days, &v)?,
            Err(reason) => {
                status = "failed";
                failure = Some(json!({"probe_day":day,"reason":reason}));
                break;
            }
        }
        if [30, 90, 180, 365].contains(&day) {
            write_json(
                &out.join(format!("checkpoint-{day}.json")),
                &state_json(&s)?,
            )?;
        }
        if day % 30 == 0 {
            eprintln!(
                "{id}: day {day}, events {cursor}, {:.1}s",
                timer.elapsed().as_secs_f64()
            );
        }
    }
    trace.finish().map_err(err)?;
    days.finish().map_err(err)?;
    // A fresh full recurrence uses the same Kernel implementation; explicitly NOT independent.
    let mut rebuilt = Adapter.genesis(&scope()).map_err(err)?;
    let verify = Instant::now();
    let mut replay_status = "passed";
    let mut replay_failure = None;
    for e in &actuals {
        match Adapter.advance(&rebuilt, e, &scope()) {
            Ok(next) => rebuilt = next,
            Err(reason) => {
                replay_failure = Some(format!("{reason:?}"));
                replay_status = "failed";
                break;
            }
        }
    }
    if !same(&s, &rebuilt)? {
        replay_status = "failed";
    }
    let rename_control = if matches!(run.world.as_str(), "W6" | "W7") {
        let mut renamed = genesis()?;
        let mut fault = None;
        for event in &actuals {
            let mut event = event.clone();
            event.person = rename_id(&event.person, false);
            match advance_named(&renamed, &event, true) {
                Ok(next) => renamed = next,
                Err(e) => {
                    fault = Some(e);
                    break;
                }
            }
        }
        let mut mapped = state_json(&renamed)?;
        inverse_names(&mut mapped);
        json!({"status":if fault.is_none() && mapped == state_json(&s)? { "passed" } else { "failed" },"failure":fault,"scope":"full committed prefix, inverse-mapped full state; U/V to X/Y, event/group IDs renamed"})
    } else {
        json!({"status":"not_applicable"})
    };
    let witness = if let Some(f) = &failure {
        if let Some(ordinal) = f["ordinal"].as_u64() {
            let e: Event = serde_json::from_value(f["input"].clone()).map_err(err)?;
            let again = advance(&rebuilt, &e).err();
            Some(
                json!({"shortest_sequential_failure_prefix_events":ordinal,"same_failure_replayed":again.as_ref().is_some_and(|reason|f["reason"].as_str()==Some(reason.as_str())),"replayed_reason":again,"basis":"all earlier prefixes committed; full original event table and trace retained; no event deletion"}),
            )
        } else {
            None
        }
    } else {
        None
    };
    write_json(
        &out.join("result.json"),
        &json!({"id":id,"label":"第一版对照 / 非正式结论","status":status,"failure":failure,"committed_events":cursor,"expected_events":input.len(),"seconds":timer.elapsed().as_secs_f64(),"event_transition_seconds":slow,"fresh_recurrence_seconds":verify.elapsed().as_secs_f64(),"same_implementation_recompute":replay_status,"recompute_failure":replay_failure,"rename_negative_control":rename_control,"failure_witness":witness,"independent_second_implementation":"unavailable","p526_replay":"mechanical test coverage only; full prefix audit replay not executed","analysis":"pending"}),
    )
}
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let result = match args.as_slice() {
        [_, mode, root] if mode == "prepare" => prepare(Path::new(root)),
        [_, mode, checkpoint, day, ordinal, hours, seed, output] if mode == "continue" => (|| {
            if cfg!(debug_assertions) {
                return Err("release required".into());
            }
            let s = experiment_world::continuation::restore(
                serde_json::from_slice(&fs::read(checkpoint).map_err(err)?).map_err(err)?,
            )?;
            let result = experiment_world::continuation::continuation(
                &s,
                day.parse().map_err(err)?,
                ordinal.parse().map_err(err)?,
                hours.parse().map_err(err)?,
                seed.parse().map_err(err)?,
            );
            write_json(
                Path::new(output),
                &match result {
                    Ok(v) => v,
                    Err(e) => json!({"status":"failed","reason":e}),
                },
            )
        })(
        ),
        [_, mode, package, id, out] if mode == "run" => {
            execute(Path::new(package), id, Path::new(out))
        }
        _ => Err(
            "usage: experiment-world prepare NEW_PACKAGE | run FROZEN_PACKAGE RUN_ID NEW_OUTPUT"
                .into(),
        ),
    };
    if let Err(e) = result {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
