use crate::{Result, error::require};
use core_types::contract::Limits;
use std::time::Duration;

/// Explicit resource limits, supplied by the versioned deployment profile.
#[derive(Clone, Copy)]
pub struct StoreConfig {
    pub encoding: Limits,
    pub busy_timeout: Duration,
    pub max_pending_records: u64,
    pub max_pending_bytes: u64,
    pub max_object_bytes: u64,
    pub max_store_bytes: u64,
    pub max_sql_pages: u32,
}
impl StoreConfig {
    pub(crate) fn validate(self) -> Result<()> {
        require(
            self.encoding.max_bytes > 0
                && self.encoding.max_items > 0
                && self.encoding.max_depth > 0,
            "encoding limits must be positive",
        )?;
        require(
            !self.busy_timeout.is_zero() && self.busy_timeout.as_millis() <= i32::MAX as u128,
            "busy wait must be bounded and positive",
        )?;
        require(
            self.max_pending_records > 0
                && self.max_pending_bytes > 0
                && self.max_object_bytes > 0
                && self.max_store_bytes >= self.max_object_bytes
                && self.max_sql_pages > 0,
            "storage limits must be positive",
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaultPoint {
    ObjectWritten,
    FileSynced,
    Renamed,
    DirectorySynced,
    GStaged,
    CStaged,
    LedgerWritten,
    BeforePublish,
    SqlCommitted,
    AfterPublish,
}
#[derive(Clone, Copy, Debug, Default)]
pub enum Fault {
    #[default]
    None,
    /// An actual process exit, deliberately bypassing Rust destructors.
    Crash(FaultPoint),
    /// Portable ENOSPC-like failure at an object durability boundary.
    DiskFull(FaultPoint),
}
impl Fault {
    pub(crate) fn hit(self, at: FaultPoint) -> Result<()> {
        match self {
            Self::Crash(p) if p == at => std::process::exit(86),
            Self::DiskFull(p) if p == at => {
                Err(std::io::Error::from(std::io::ErrorKind::StorageFull).into())
            }
            _ => Ok(()),
        }
    }
}
