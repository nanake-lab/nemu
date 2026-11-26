//! ADR 0049. Check the parentless bundle before resolving its resources and local authority.
use crate::WireMessage;
use crate::contract::{Limits, Result, Schema, require};
use crate::generated::{
    BirthNameRecord, EmptyPrefixKind, GenesisManifestV2, GenesisManifestV3, InitialPublication,
    InitialRuntimeRoot, Lineage, LineageHeaderV2, LocalUserBinding,
};
use crate::scalar::hex;
use serde_json::Value;
use sha2::{Digest, Sha256};

fn runtime(q: &InitialRuntimeRoot) -> Result<()> {
    let env = &q.environment;
    for (stage, resource) in [
        (q.irl.as_ref(), Some(&env.irl)),
        (q.appraiser.as_ref(), env.nature_appraiser.as_ref()),
        (q.matcher.as_ref(), env.situation_matcher.as_ref()),
    ] {
        require(
            stage.is_none() || stage == resource,
            "initial stage resource mismatch",
        )?;
    }
    for stage in [&q.fast, &q.growth, &q.dynamics, &q.compiler]
        .into_iter()
        .flatten()
    {
        require(
            env.implementations.contains(stage),
            "unbound initial implementation",
        )?;
    }
    require(
        (q.fast.is_none() && q.growth.is_none() && q.dynamics.is_none())
            || env.law_version.is_some(),
        "initial law missing",
    )?;
    require(
        q.compiler.is_none() || env.direction_set.is_some(),
        "initial directions missing",
    )
}

fn birth(b: &BirthNameRecord) -> Result<()> {
    require(
        matches!(
            b.manifest.object_type.as_str(),
            "echo.genesis-v2" | "echo.genesis-v3"
        ) && b.manifest.resource_version == "1",
        "birth manifest type or schema version",
    )?;
    require(
        b.signature_evidence.object_type == "nemu.storage.raw-bytes-v1"
            && b.signature_evidence.resource_version == "1"
            && b.signature_evidence.byte_length.0 > 0,
        "birth signature evidence",
    )
}

fn publication(p: &InitialPublication) -> Result<()> {
    let q = &p.q;
    runtime(q)?;
    require(
        matches!(
            p.x.object_type.as_str(),
            "echo.fast-state-v2" | "echo.fast-state-v3"
        ) && p.r.object_type == "echo.r-snapshot-v3"
            && matches!(
                p.theta.object_type.as_str(),
                "echo.theta-snapshot-v2" | "echo.theta-snapshot-v3"
            ),
        "initial snapshot type",
    )?;
    match p.header.lineage {
        Lineage::REHEARSAL => require(
            q.growth.is_none() && q.dynamics.is_none(),
            "rehearsal growth",
        )?,
        Lineage::LIFE | Lineage::EXPERIMENT => require(
            [
                &q.irl,
                &q.appraiser,
                &q.matcher,
                &q.fast,
                &q.growth,
                &q.dynamics,
                &q.compiler,
            ]
            .iter()
            .all(|stage| stage.is_some()),
            "incomplete initial stages",
        )?,
    }
    match (&p.header.lineage, &p.birth, &p.empty_history) {
        (Lineage::LIFE, Some(b), None) => {
            birth(b)?;
            require(
                p.header.genesis_manifest_digest.as_ref() == Some(&b.manifest.digest),
                "birth manifest digest",
            )
        }
        (Lineage::REHEARSAL | Lineage::EXPERIMENT, None, Some(empty)) => require(
            empty.kind == EmptyPrefixKind::EMPTY_HISTORY,
            "initial H requires empty history",
        ),
        _ => Err("wrong initial history for lineage".into()),
    }
}

/// Run after either wire decoder, before consuming an initial object (I02/I18/I29).
pub fn check_initial_message(name: &str, value: &Value, limits: Limits) -> Result<()> {
    if !matches!(
        name,
        "EmptyPrefixRoot" | "InitialRuntimeRoot" | "BirthNameRecord" | "InitialPublication"
    ) {
        return Ok(());
    }
    let v = Schema::shared_builtin()?.normalize(name, value, limits, 0)?;
    let err = |e: serde_json::Error| e.to_string();
    match name {
        "EmptyPrefixRoot" => Ok(()),
        "InitialRuntimeRoot" => runtime(&serde_json::from_value(v).map_err(err)?),
        "BirthNameRecord" => birth(&serde_json::from_value(v).map_err(err)?),
        _ => publication(&serde_json::from_value(v).map_err(err)?),
    }
}

/// The expected identity, header and stage bindings come from the trusted local owners.
/// The writer must still hold the store capability and recheck the head when publishing.
pub fn check_initial_store_binding(
    root: &InitialPublication,
    store_id: &str,
    branch_id: &str,
    header: &LineageHeaderV2,
    connected: &InitialRuntimeRoot,
    limits: Limits,
) -> Result<()> {
    root.canonical_bytes(limits)?;
    publication(root)?;
    require(
        root.store_id == store_id && root.branch_id == branch_id && root.header == *header,
        "wrong initial store, branch or lineage",
    )?;
    require(
        root.q.canonical_bytes(limits)? == connected.canonical_bytes(limits)?,
        "initial stages differ from connected resources",
    )
}

/// W14 supplies its authenticated birth session, the original bytes, and its trusted
/// signature verifier. This check issues no storage/instance capability.
pub fn verify_birth_material(
    record: &BirthNameRecord,
    authenticated: &LocalUserBinding,
    manifest: &[u8],
    signature: &[u8],
    limits: Limits,
    verify_signature: impl FnOnce(&LocalUserBinding, &[u8], &[u8]) -> Result<()>,
) -> Result<()> {
    record.canonical_bytes(limits)?;
    birth(record)?;
    require(record.caretaker == *authenticated, "wrong birth session")?;
    require(
        manifest.len() <= limits.max_bytes
            && manifest.len() as u64 == record.manifest.byte_length.0
            && signature.len() <= limits.max_bytes
            && signature.len() as u64 == record.signature_evidence.byte_length.0,
        "birth material length",
    )?;
    require(
        Sha256::digest(signature).as_slice() == record.signature_evidence.digest.0,
        "birth signature digest",
    )?;
    let schema = Schema::shared_builtin()?;
    let is_v2 = record.manifest.object_type == "echo.genesis-v2";
    let name = if is_v2 {
        GenesisManifestV2::NAME
    } else {
        GenesisManifestV3::NAME
    };
    let value = schema.decode_canonical(name, manifest, limits, 0)?;
    require(
        schema.digest(name, &value, limits)? == hex(&record.manifest.digest.0),
        "birth manifest digest",
    )?;
    require(
        value["caretaker_id"] == authenticated.principal_id,
        "manifest caretaker mismatch",
    )?;
    let err = |e: serde_json::Error| e.to_string();
    if is_v2 {
        super::check_genesis(&serde_json::from_value(value).map_err(err)?)?;
    } else {
        super::check_genesis_v3(&serde_json::from_value(value).map_err(err)?)?;
    }
    verify_signature(authenticated, manifest, signature)
}
