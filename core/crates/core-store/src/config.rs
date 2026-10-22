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
