use crate::{deployment, logging, signal};
use core_store::{Rehearsal, Store};
use core_types::{generated::*, scalar::DecimalI64};
use gateway::{
    Grants, TerminalToken,
    gateway::{Gateway, Installation},
    hooks::{CoreHandler, Decision},
};
use std::{
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tracing::level_filters::LevelFilter;

struct WaitingForRoot;

impl CoreHandler<Rehearsal> for WaitingForRoot {
    fn authorize_event(
        &mut self,
        _: &Store<Rehearsal>,
        _: &TerminalSession,
        _: &TerminalEvent,
        _: &SourceMetadata,
    ) -> Decision<Grants> {
        // HOST-01-user-text: pairing authenticates an installation. The source
        // authority predicate for an assertion from a person is still missing.
        Err(IngressReason::CAPABILITY_DENIED)
    }
    // The trait denies all controls and process_next returns no commit results.
}

fn options(args: impl Iterator<Item = String>) -> Result<LevelFilter, &'static str> {
    let mut lineage = None;
    let mut level = None;
    let mut args = args;
    while let Some(flag) = args.next() {
        let value = args.next().ok_or("missing option value")?;
        match flag.as_str() {
            "--lineage" if lineage.is_none() && value == "rehearsal" => lineage = Some(()),
            "--lineage" => return Err("only a single rehearsal lineage is supported"),
            "--log-level" if level.is_none() => {
                level = Some(value.parse().map_err(|_| "invalid log level")?);
            }
            _ => return Err("unknown or duplicate option"),
        }
    }
    Ok(level.unwrap_or(LevelFilter::INFO))
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let level = options(std::env::args().skip(1))?;
    logging::install(level)?;
    let base = PathBuf::from(std::env::var_os("LOCALAPPDATA").ok_or("LOCALAPPDATA missing")?);
    if !base.is_absolute() {
        return Err("LOCALAPPDATA must be absolute".into());
    }
    let header = LineageHeaderV2 {
        schema_version: 1,
        lineage: Lineage::REHEARSAL,
        created_at_ns: DecimalI64(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)?
                .as_nanos()
                .try_into()?,
        ),
        genesis_manifest_digest: None,
        world_id: None,
        world_seed: None,
    };
    let (storage, config) = deployment::settings();
    let store = Store::<Rehearsal>::startup(header, "rehearsal", storage)
        .map_err(|_| "rehearsal store startup failed")?;
    let status = if store
        .head()
        .map_err(|_| "cannot read rehearsal head")?
        .0
        .is_none()
    {
        "waiting_for_initial_root"
    } else {
        "runtime_handler_pending"
    };
    // The exclusive store writer serializes first-time provisioning. Only a
    // missing path may be created. Existing inaccessible/corrupt tokens fail closed.
    let path = base.join("Nemu").join("terminal.token");
    let token = match std::fs::symlink_metadata(&path) {
        Ok(_) => TerminalToken::load_protected(&path)?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            TerminalToken::create_protected(&path)?
        }
        Err(_) => return Err("cannot inspect terminal token path".into()),
    };
    use IngressKind::*;
    let install = Installation::new(
        "desktop".into(),
        token,
        vec![
            INGRESS_EVENT,
            INGRESS_UPLOAD,
            INGRESS_QUERY,
            INGRESS_APPROVAL,
            INGRESS_CANCEL,
        ],
    );
    let mut gateway = Gateway::new(store, WaitingForRoot, vec![install], config)?;
    signal::install()?;
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(gateway.serve(async {
            // serve polls this future only after binding the exclusive SID pipe.
            tracing::info!("rehearsal gateway listening");
            let mut ticks = tokio::time::interval(Duration::from_secs(2));
            ticks.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                ticks.tick().await;
                if signal::requested() {
                    break;
                }
                println!("HEARTBEAT rehearsal {status}");
            }
        }))?;
    tracing::info!("rehearsal gateway stopped");
    Ok(())
}
