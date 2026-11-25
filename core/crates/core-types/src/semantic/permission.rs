//! ADR 0047. Local message consistency. Issuance and consumption belong to P5-41.
use crate::contract::{Limits, Result, Schema, require};
use crate::scalar::{hex, unhex};
use serde_json::Value;
use sha2::{Digest, Sha256};

// Normalization has checked signed/unsigned 64-bit ranges; i128 compares both exactly.
fn integer(v: &Value) -> i128 {
    v.as_str()
        .expect("normalized integer")
        .parse()
        .expect("64-bit integer")
}

fn scope(s: &Value) -> Result<()> {
    if let Some(amount) = s.get("wallet_minor_units") {
        require(integer(amount) > 0, "empty wallet transfer")?;
    }
    Ok(())
}

fn budget(b: &Value) -> Result<()> {
    require(
        integer(&b["charge"]) <= integer(&b["limit"]),
        "charge exceeds grant",
    )
}

fn trust(t: &Value) -> Result<()> {
    require(t["tier"] == 2 || t["tier"] == 3, "only T1/T2 trust")?;
    scope(&t["scope"])
}

fn command(c: &Value) -> Result<()> {
    if c["kind"] == 3 {
        require(
            c["alarm_record"]["object_type"] == "echo.system-maintenance-record",
            "wrong alarm record",
        )?;
        require(
            c.get("replacement_law").is_some() == (c["verdict"] == 2),
            "replacement law",
        )?;
        require(c["replacement_law"] != c["law_version"], "law unchanged")?;
    }
    Ok(())
}

fn management(m: &Value) -> Result<()> {
    command(&m["command"])?;
    require(
        m["entry"] == 2 || m["command"]["kind"] != 3,
        "review requires echoctl",
    )
}

fn grant(g: &Value) -> Result<()> {
    let s = &g["scope"];
    scope(s)?;
    budget(&g["budget"])?;
    require(
        integer(&g["issued_at_ns"]) < integer(&g["expires_at_ns"]),
        "empty validity interval",
    )?;
    if let Some(t) = g.get("trust") {
        trust(t)?;
        require(
            t["scope"] == *s
                && t["tier"] == g["tier"]
                && t["revocation_version"] == g["revocation_version"],
            "trust binding",
        )?;
    }
    let consent = g.get("consent");
    if let Some(c) = consent {
        require(
            c["action_id"] == g["action_id"]
                && c["payload_digest"] == g["payload_digest"]
                && integer(&c["confirmed_at_ns"]) <= integer(&g["issued_at_ns"]),
            "consent binding",
        )?;
    }
    if s["kind"] == 2 {
        if s["reversible"] == false {
            let tier = &s["desktop"]["irreversible"];
            require(
                *tier == 3 || (*tier == 2 && consent.is_some()),
                "desktop policy",
            )?;
        }
    } else if g["tier"] == 4
        || (s["kind"] == 1 && (g["tier"] == 2 || g["tier"] == 3) && g.get("trust").is_none())
    {
        require(consent.is_some(), "missing consent")?;
    }
    Ok(())
}

fn result(m: &Value) -> Result<()> {
    if let Some(record) = m.get("management_commit") {
        require(
            matches!(record["kind"].as_u64(), Some(1..=3)),
            "invalid maintenance result",
        )?;
    }
    Ok(())
}

pub fn check_permission_message(name: &str, value: &Value, limits: Limits) -> Result<()> {
    if !matches!(
        name,
        "PermissionResult"
            | "TerminalResponseV2"
            | "PermissionContext"
            | "ConnectorScope"
            | "ExecutionBudget"
            | "TrustMemory"
            | "ExecutionCredential"
            | "GrowthCommandV2"
            | "GrowthManagementRequest"
            | "TerminalRequestV2"
            | "ActionRequestV2"
    ) {
        return Ok(());
    }
    let v = Schema::shared_builtin()?.normalize(name, value, limits, 0)?;
    match name {
        "PermissionResult" => result(&v),
        "TerminalResponseV2" => {
            let m = v.get("management");
            require(
                (v["reply_to"] == 6) == m.is_some(),
                "missing or unexpected management reply",
            )?;
            if let Some(m) = m {
                result(m)?;
                require(
                    v["request_id"] == m["request_id"]
                        && ((v["kind"] == 4 && m.get("management_commit").is_some())
                            || (v["kind"] == 5 && m.get("denied").is_some())),
                    "wrong management reply",
                )?;
            }
            Ok(())
        }
        "PermissionContext" => match v.get("local_execution") {
            Some(g) => grant(g),
            None => super::check_speak_think_message(
                "RemoteCognitiveReceipt",
                &v["remote_text"],
                limits,
            ),
        },
        "ConnectorScope" => scope(&v),
        "ExecutionBudget" => budget(&v),
        "TrustMemory" => trust(&v),
        "ExecutionCredential" => grant(&v),
        "GrowthCommandV2" => command(&v),
        "GrowthManagementRequest" => management(&v),
        "TerminalRequestV2" if v["kind"] == 6 => {
            management(&v["management"])?;
            require(
                v["session_id"] == v["management"]["user"]["session_id"],
                "wrong session",
            )
        }
        "ActionRequestV2" => {
            let g = &v["grant"];
            grant(g)?;
            let bytes = unhex(v["action_payload"].as_str().expect("normalized bytes"))?;
            require(
                v["action_id"] == g["action_id"]
                    && v["expires_at_ns"] == g["expires_at_ns"]
                    && g["payload_digest"] == hex(&Sha256::digest(bytes)),
                "dispatch binding",
            )
        }
        _ => Ok(()),
    }
}
