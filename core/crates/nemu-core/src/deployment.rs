use core_store::StoreConfig;
use core_types::{
    contract::Limits,
    generated::{IngressAuditBudget, TerminalBudget},
};
use gateway::{framing::FrameConfig, gateway::GatewayConfig};
use std::time::Duration;

pub fn settings() -> (StoreConfig, GatewayConfig) {
    // Encoding and wire limits follow the W18 native service. W03's store
    // window also accommodates the outstanding events while awaiting a root.
    let encoding = Limits {
        max_bytes: 4 << 20,
        max_items: 4096,
        max_depth: 64,
    };
    let store = StoreConfig {
        encoding,
        busy_timeout: Duration::from_millis(30),
        max_pending_records: 32,
        max_pending_bytes: 1 << 20,
        max_object_bytes: 4 << 20,
        max_store_bytes: 64 << 20,
        max_sql_pages: 8192,
    };
    let gateway = GatewayConfig {
        frames: FrameConfig {
            encoding,
            deadline: Duration::from_secs(10),
        },
        budget: TerminalBudget {
            schema_version: 1,
            version: "rehearsal-terminal-1".into(),
            max_frame_bytes: 16384,
            max_chunk_bytes: 2048,
            max_object_bytes: 8192,
            max_uploads: 2,
            max_upload_bytes: 32768,
        },
        audit: IngressAuditBudget {
            schema_version: 1,
            version: "rehearsal-audit-1".into(),
            max_records: 100,
            max_bytes: 65536,
        },
        max_connections: 4,
        max_pending_requests: store.max_pending_records as usize,
        poll_interval: Duration::from_millis(5),
        upload_lifetime: Duration::from_secs(10),
        // Cover a full day of desktop use before requiring the terminal to reauthenticate.
        session_lifetime: Duration::from_secs(12 * 60 * 60),
    };
    (store, gateway)
}
