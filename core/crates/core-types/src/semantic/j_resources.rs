//! ADR 0036. Boundary consistency; never commit or calibration authority.
use crate::contract::{Limits, Result, Schema, require};
use serde_json::Value;

fn list<'a>(v: &'a Value, k: &str) -> &'a [Value] {
    v[k].as_array().expect("normalized list")
}
fn number(v: &Value, k: &str) -> f64 {
    v[k].as_f64().expect("normalized double")
}
fn time(v: &Value, k: &str) -> u64 {
    v[k].as_str().expect("uint64").parse().expect("uint64")
}
fn tag(v: &Value, k: &str, expected: &str) -> Result<()> {
    require(
        v.get(k).is_none() || v[k]["object_type"] == expected,
        "J reference type",
    )
}
fn add_bound(a: f64, b: f64) -> Result<f64> {
    let (hi, lo) = if a >= b { (a, b) } else { (b, a) };
    let sum = hi + lo;
    require(sum.is_finite(), "beta overflow")?;
    let sum = if lo - (sum - hi) > 0.0 {
        sum.next_up()
    } else {
        sum
    };
    require(sum.is_finite(), "beta overflow")?;
    Ok(sum)
}
pub fn check_j_envelope(envelope: &Value, receipt: &Value, limits: Limits) -> Result<()> {
    let schema = Schema::builtin()?;
    let e = schema.normalize("ModulationEnvelopeV2", envelope, limits, 0)?;
    let r = schema.normalize("JEffectReceipt", receipt, limits, 0)?;
    check_j_message("JEffectReceipt", &r, limits)?;
    require(
        e["effect_receipt_digest"] == schema.digest("JEffectReceipt", &r, limits)?,
        "receipt digest",
    )?;
    for k in [
        "snapshot_id",
        "projected_at_ns",
        "x_digest",
        "r_digest",
        "theta_digest",
        "carrier",
        "model",
        "tokenizer",
        "direction_set",
        "total_bound",
    ] {
        require(e[k] == r[k], "envelope binding")?;
    }
    let ds = list(&r, "directions");
    for (i, k) in ["connection", "empathy", "curiosity", "achievement"]
        .iter()
        .enumerate()
    {
        require(
            e["nature_coefficients"][k] == ds[i]["coefficient"],
            "nature coefficient",
        )?;
    }
    let learned: Vec<_> = ds[4..].iter().filter(|d| d["status"] == 1).collect();
    let actual = list(&e, "learned_coefficients");
    require(learned.len() == actual.len(), "learned coefficients")?;
    for (d, a) in learned.iter().zip(actual) {
        require(
            d["tendency_id"] == a["tendency_id"] && d["coefficient"] == a["coefficient"],
            "learned coefficient",
        )?;
    }
    Ok(())
}
fn ordered(items: &[Value]) -> Result<()> {
    require(items.len() >= 4, "four nature directions required")?;
    for (i, d) in items[..4].iter().enumerate() {
        require(d["kind"] == i + 1, "nature order")?;
    }
    let mut previous: Option<&str> = None;
    for d in &items[4..] {
        require(d["kind"] == 5, "learned kind")?;
        let id = d["tendency_id"].as_str().ok_or("tendency identity")?;
        if let Some(p) = previous {
            require(
                super::tendency_order(p, id).is_lt(),
                "learned order/duplicate",
            )?;
        }
        previous = Some(id);
    }
    Ok(())
}

pub fn check_j_message(name: &str, value: &Value, limits: Limits) -> Result<()> {
    if !matches!(
        name,
        "JHookBinding"
            | "JDirectionVector"
            | "JDirection"
            | "JDirectionSet"
            | "JValidatedDirection"
            | "JDirectionValidation"
            | "BodyReadoutParameters"
            | "JDirectionUse"
            | "JEffectReceipt"
            | "GenesisManifestV3"
    ) {
        return Ok(());
    }
    let schema = Schema::builtin()?;
    let v = schema.normalize(name, value, limits, 0)?;
    match name {
        "JDirectionVector" => require(!list(&v, "values").is_empty(), "empty vector")?,
        "JDirection" | "JDirectionUse" => {
            let learned = v["kind"] == 5;
            require(
                v.get("tendency_id").is_some() == learned,
                "tendency identity",
            )?;
            if name == "JDirection" {
                require(v.get("concept").is_some() == learned, "concept identity")?;
                tag(&v, "vector", "echo.j-direction-vector")?;
            } else {
                let included = v["status"] == 1;
                require(learned || included, "nature cannot be excluded")?;
                require(
                    ["direction", "validation", "beta", "coefficient"]
                        .iter()
                        .all(|k| v.get(k).is_some() == included),
                    "use presence",
                )?;
                tag(&v, "direction", "echo.j-direction-vector")?;
                tag(&v, "validation", "echo.j-direction-validation")?;
                if included {
                    require(
                        number(&v, "coefficient").abs() <= number(&v, "beta"),
                        "coefficient budget",
                    )?;
                }
            }
        }
        "JDirectionSet" | "JDirectionValidation" => {
            let ds = list(&v, "directions");
            if name == "JDirectionSet" {
                ordered(ds)?;
            } else {
                tag(&v, "direction_set", "echo.j-direction-set")?;
                require(ds.len() >= 4, "validation list")?;
            }
            for d in ds {
                check_j_message(
                    if name == "JDirectionSet" {
                        "JDirection"
                    } else {
                        "JValidatedDirection"
                    },
                    d,
                    limits,
                )?;
            }
        }
        "JValidatedDirection" => {
            tag(&v, "vector", "echo.j-direction-vector")?;
            require(
                (number(&v, "measured_norm") - 1.0).abs() <= number(&v, "norm_tolerance"),
                "unit norm",
            )?;
        }
        "JEffectReceipt" => {
            require(
                time(&v, "projected_at_ns") >= time(&v, "anchor_time_ns"),
                "time before anchor",
            )?;
            for (k, t) in [
                ("direction_set", "echo.j-direction-set"),
                ("validation", "echo.j-direction-validation"),
                ("body_parameters", "echo.body-readout-parameters"),
                ("genesis", "echo.genesis-v3"),
            ] {
                tag(&v, k, t)?;
            }
            ordered(list(&v, "directions"))?;
            let mut total = 0.0;
            for d in list(&v, "directions") {
                check_j_message("JDirectionUse", d, limits)?;
                if d["status"] == 1 {
                    total = add_bound(total, number(d, "beta"))?;
                    if v["mode"] == 2 {
                        require(number(d, "coefficient") == 0.0, "rehearsal coefficient")?;
                    }
                }
            }
            require(total == number(&v, "total_bound"), "total bound")?;
            if v["mode"] == 2 {
                require(
                    v["body"]["facing"] == 0.5
                        && v["body"]["tension"] == 0.0
                        && v["body"]["energy"] == 0.5,
                    "rehearsal body",
                )?;
            }
        }
        "GenesisManifestV3" => {
            tag(&v, "direction_set", "echo.j-direction-set")?;
            tag(&v, "direction_validation", "echo.j-direction-validation")?;
            tag(&v, "body_readout", "echo.body-readout-parameters")?;
        }
        _ => {}
    }
    Ok(())
}

/// Resolved immutable objects for checking, not a new wire schema or authority store.
/// Caller must separately verify committed state, R+ completeness and actual evidence.
pub struct JResolvedResources<'a> {
    pub direction_set: &'a Value,
    pub validation: &'a Value,
    pub body: &'a Value,
    pub receipt: &'a Value,
    pub genesis: &'a Value,
    pub vectors: &'a [Value],
}
pub fn check_j_resources(
    input: JResolvedResources<'_>,
    formal: bool,
    limits: Limits,
) -> Result<()> {
    let schema = Schema::builtin()?;
    let pairs = [
        ("JDirectionSet", input.direction_set),
        ("JDirectionValidation", input.validation),
        ("BodyReadoutParameters", input.body),
        ("JEffectReceipt", input.receipt),
        ("GenesisManifestV3", input.genesis),
    ];
    let mut values = Vec::new();
    for (name, value) in pairs {
        check_j_message(name, value, limits)?;
        values.push(schema.normalize(name, value, limits, 0)?);
    }
    let (s, v, b, r, g) = (&values[0], &values[1], &values[2], &values[3], &values[4]);
    super::check_genesis_v3(&serde_json::from_value(g.clone()).map_err(|e| e.to_string())?)?;
    let bound = |reference: &Value, name: &str, value: &Value| -> Result<()> {
        require(
            reference["object_type"] == schema.message(name)?.tag,
            "resource type",
        )?;
        require(
            reference["digest"] == schema.digest(name, value, limits)?,
            "resource digest",
        )?;
        require(
            time(reference, "byte_length")
                == schema.encode_canonical(name, value, limits)?.len() as u64,
            "resource length",
        )?;
        if value.get("version").is_some() {
            require(
                reference["resource_version"] == value["version"],
                "resource version",
            )?;
        } else {
            require(
                reference["resource_version"] == "1",
                "resource schema version",
            )?;
        }
        Ok(())
    };
    for reference in [&v["direction_set"], &r["direction_set"]] {
        bound(reference, "JDirectionSet", s)?;
    }
    bound(&r["validation"], "JDirectionValidation", v)?;
    for reference in [&r["body_parameters"], &g["body_readout"]] {
        bound(reference, "BodyReadoutParameters", b)?;
    }
    bound(&r["genesis"], "GenesisManifestV3", g)?;
    for key in ["carrier", "model", "tokenizer"] {
        require(r[key] == s["hook"][key], "carrier binding")?;
    }
    require(r["body"]["version"] == b["version"], "body version")?;
    if formal {
        require(
            v["fixture"] == false && r["mode"] == 1,
            "fixture/rehearsal not formal",
        )?;
        require(
            s["hook"]["prefill_mask"] != 3 && s["hook"]["decode_mask"] == 1,
            "formal prediction positions",
        )?;
    }
    let ds = list(s, "directions");
    let checks = list(v, "directions");
    require(
        ds.len() == checks.len() && ds.len() == input.vectors.len(),
        "resource lists",
    )?;
    for ((d, checked), vector) in ds.iter().zip(checks).zip(input.vectors) {
        check_j_message("JDirectionVector", vector, limits)?;
        let vector = schema.normalize("JDirectionVector", vector, limits, 0)?;
        require(
            d["vector"] == checked["vector"],
            "validation direction order",
        )?;
        bound(&d["vector"], "JDirectionVector", &vector)?;
        require(
            list(&vector, "values").len() as u64 == s["hook"]["width"].as_u64().expect("width"),
            "vector width",
        )?;
        let mut total = 0.0;
        for x in list(&vector, "values") {
            let x = x.as_f64().expect("double");
            total += x * x;
            require(total.is_finite(), "norm overflow")?;
        }
        require(
            total.sqrt() == number(checked, "measured_norm"),
            "measured norm",
        )?;
        require(
            number(d, "beta") <= number(checked, "max_beta"),
            "single beta budget",
        )?;
    }
    require(
        number(r, "total_bound") <= number(v, "combination_max_beta"),
        "combination budget",
    )?;
    for use_ in list(r, "directions") {
        if use_["status"] != 1 {
            continue;
        }
        let found: Vec<_> = ds
            .iter()
            .filter(|d| d["kind"] == use_["kind"] && d["tendency_id"] == use_["tendency_id"])
            .collect();
        require(found.len() == 1, "included identity")?;
        require(
            use_["direction"] == found[0]["vector"] && use_["beta"] == found[0]["beta"],
            "included binding",
        )?;
        require(use_["validation"] == r["validation"], "included validation")?;
    }
    Ok(())
}
