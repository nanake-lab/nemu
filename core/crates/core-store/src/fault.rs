use crate::Result;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaultPoint {
    ObjectWritten,
    FileSynced,
    Renamed,
    DirectorySynced,
    ReuseFileSyncBefore,
    ReuseFileSynced,
    ReuseDirectorySyncBefore,
    ReuseDirectorySynced,
    GStaged,
    CStaged,
    LedgerWritten,
    BeforePublish,
    SqlCommitted,
    AfterPublish,
    IngressAllocated,
    AuditBeforeSql,
    AuditSqlCommitted,
    MigrationBeforeSql,
    MigrationSqlCommitted,
    IngressBeforeSql,
    IngressSqlCommitted,
    IngressAcknowledged,
    ReceiptWritten,
    ReceiptFileSyncBefore,
    ReceiptFileSynced,
    ReceiptRenameBefore,
    ReceiptRenamed,
    ReceiptTargetSyncBefore,
    ReceiptTargetSynced,
    ReceiptDirectorySyncBefore,
    ReceiptDirectorySynced,
    InitRootCreated,
    InitRootSynced,
    InitObjectsCreated,
    InitObjectsSynced,
    InitSqlCommitted,
    InitDbDirectorySynced,
    ReceiptRootCreated,
    ReceiptKindCreated,
    ReceiptKindSynced,
    ReceiptRootSynced,
    ReceiptParentSynced,
    GcDeleted,
    GcDirectorySyncBefore,
    GcDirectorySynced,
}

#[derive(Clone, Debug)]
pub struct NthFault {
    point: FaultPoint,
    occurrence: usize,
    object_type: Option<String>,
    seen: Arc<AtomicUsize>,
}
#[derive(Clone, Debug, Default)]
pub enum Fault {
    #[default]
    None,
    /// Actual exit bypasses destructors; only use in freshly owned test sandboxes.
    Crash(FaultPoint),
    DiskFull(FaultPoint),
    Nth(NthFault),
}
impl Fault {
    /// Select one operation, optional generated object tag, and a 1-based visit.
    /// Clones share the counter, so nested R writes do not reset the occurrence.
    pub fn crash_nth(point: FaultPoint, occurrence: usize, object_type: Option<&str>) -> Self {
        assert!(occurrence > 0, "fault occurrences are 1-based");
        Self::Nth(NthFault {
            point,
            occurrence,
            object_type: object_type.map(str::to_owned),
            seen: Arc::new(AtomicUsize::new(0)),
        })
    }
    pub(crate) fn hit(&self, at: FaultPoint) -> Result<()> {
        self.hit_object(at, None)
    }
    pub(crate) fn hit_object(&self, at: FaultPoint, object_type: Option<&str>) -> Result<()> {
        let crash = match self {
            Self::Crash(p) => *p == at,
            Self::DiskFull(p) if *p == at => {
                return Err(std::io::Error::from(std::io::ErrorKind::StorageFull).into());
            }
            Self::Nth(n)
                if n.point == at
                    && n.object_type
                        .as_deref()
                        .is_none_or(|t| Some(t) == object_type) =>
            {
                n.seen.fetch_add(1, Ordering::SeqCst) + 1 == n.occurrence
            }
            _ => false,
        };
        if crash {
            use std::io::Write;
            println!("W02_INJECT {at:?} object={object_type:?} plan={self:?}");
            std::io::stdout().flush()?;
            std::process::exit(86);
        }
        Ok(())
    }
}
