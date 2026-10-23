//! Manual, synthetic second-account acceptance. Never opens existing user stores
//! or creates accounts/settings. No terminal secret is generated or transmitted.
#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use core_store::{Rehearsal, Store, StoreConfig, TemporaryRoot};
    use core_types::{
        contract::Limits,
        generated::{Lineage, LineageHeaderV2},
        scalar::DecimalI64,
    };
    use gateway::{
        framing::FrameConfig,
        pipe::{Listener, current_pipe_name},
    };
    use std::time::Duration;
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.first().is_some_and(|s| s == "deny") {
        let sid = args.get(1).ok_or("usage: acl_probe deny <owner SID>")?;
        if !sid.starts_with("S-1-5-21-")
            || !sid
                .bytes()
                .all(|b| b.is_ascii_digit() || b == b'-' || b == b'S')
        {
            return Err("expected a Windows user SID".into());
        }
        let name = format!(r"\\.\pipe\nemu-core-{sid}");
        if name == current_pipe_name()? {
            return Err("must run deny from a DIFFERENT Windows account".into());
        }
        match std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&name)
        {
            Err(error) if error.raw_os_error() == Some(5) => {
                println!("passed: different Windows SID received ERROR_ACCESS_DENIED (5)");
                return Ok(());
            }
            Err(error) => {
                return Err(format!(
                    "NOT PASSED: expected 5; actual Win32 {:?}",
                    error.raw_os_error()
                )
                .into());
            }
            Ok(_) => return Err("FAILED: different account connected".into()),
        }
    }
    if args.as_slice() != ["serve"] {
        println!("usage: acl_probe serve | acl_probe deny <owner SID>");
        return Ok(());
    }
    // No path parameter and no production opening API. Only a newly allocated
    // TemporaryRoot can supply the synthetic store to the real listener binding.
    let root = TemporaryRoot::new()?;
    let encoding = Limits {
        max_bytes: 1024 * 1024,
        max_items: 1024,
        max_depth: 32,
    };
    let config = StoreConfig {
        encoding,
        busy_timeout: Duration::from_millis(50),
        max_pending_records: 8,
        max_pending_bytes: 1024 * 1024,
        max_object_bytes: 1024 * 1024,
        max_store_bytes: 8 * 1024 * 1024,
        max_sql_pages: 1024,
    };
    let store = Store::<Rehearsal>::create_temporary(
        &root,
        "acl-probe",
        LineageHeaderV2 {
            schema_version: 1,
            lineage: Lineage::REHEARSAL,
            created_at_ns: DecimalI64(1),
            genesis_manifest_digest: None,
            world_id: None,
            world_seed: None,
        },
        "synthetic-acl-probe",
        config,
    )?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async {
        let mut listener = Listener::bind(
            &store,
            FrameConfig {
                encoding,
                deadline: Duration::from_secs(120),
            },
        )?;
        println!("synthetic listener: {}", current_pipe_name()?);
        println!(
            "Run deny from the second account now. No authentication/token exchange is performed."
        );
        match listener.accept().await {
            Err(gateway::Error::Timeout) => {
                println!("listener window ended; attach the second-account result separately")
            }
            Ok(_) => println!("a permitted account connected; this is not denial evidence"),
            Err(error) => return Err(error),
        }
        Ok::<_, gateway::Error>(())
    })?;
    Ok(())
}
#[cfg(not(windows))]
fn main() {
    println!("Windows-only manual ACL probe");
}
