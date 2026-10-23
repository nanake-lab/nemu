use crate::{Result, error::require};
use core_types::generated::Lineage;
use std::path::{Path, PathBuf};

mod sealed {
    pub trait Sealed {}
}
pub trait LineageKind: sealed::Sealed + Send + Sync + 'static {
    const KIND: Lineage;
    const DIRECTORY: &'static str;
}
pub enum Rehearsal {}
pub enum Life {}
pub enum Experiment {}
impl sealed::Sealed for Rehearsal {}
impl sealed::Sealed for Life {}
impl sealed::Sealed for Experiment {}
impl LineageKind for Rehearsal {
    const KIND: Lineage = Lineage::REHEARSAL;
    const DIRECTORY: &'static str = "rehearsal";
}
impl LineageKind for Life {
    const KIND: Lineage = Lineage::LIFE;
    const DIRECTORY: &'static str = "life";
}
impl LineageKind for Experiment {
    const KIND: Lineage = Lineage::EXPERIMENT;
    const DIRECTORY: &'static str = "experiment";
}

/// Owns a newly allocated temporary sandbox. There is no constructor from a path.
/// Tests and experiments cannot use this API to designate a user's existing store.
pub struct TemporaryRoot(tempfile::TempDir);
impl TemporaryRoot {
    pub fn new() -> Result<Self> {
        Ok(Self(
            tempfile::Builder::new().prefix("nemu-w02-").tempdir()?,
        ))
    }
    pub fn path(&self) -> &Path {
        self.0.path()
    }
    pub(crate) fn store_path(&self, name: &str) -> Result<PathBuf> {
        require(
            !name.is_empty() && name.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-'),
            "invalid temporary store name",
        )?;
        Ok(self.path().join(name))
    }
}

pub(crate) fn production_path<L: LineageKind>() -> Result<PathBuf> {
    require(
        L::KIND != Lineage::EXPERIMENT,
        "experiments require an owned temporary directory",
    )?;
    let base = std::env::var_os("LOCALAPPDATA")
        .ok_or(crate::Error::Rejected("LOCALAPPDATA is unavailable"))?;
    Ok(PathBuf::from(base)
        .join("Nemu")
        .join("core")
        .join(L::DIRECTORY))
}
