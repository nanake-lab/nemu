//! ADR 0049. A parentless publication has no input event, elapsed-time sample or parent.
use crate::{
    CommitCapability, Error, FaultPoint, PreparedObject, Rehearsal, Result, Store, error::require,
    objects::Objects, publication, schema::integer,
};
use core_types::{WireMessage, generated::*, scalar::hex, semantic::check_initial_store_binding};
use rusqlite::{Connection, OptionalExtension, params};
use std::sync::Arc;

pub(crate) const TAG: &str = "echo.initial-publication";

fn empty(kind: EmptyPrefixKind) -> EmptyPrefixRoot {
    EmptyPrefixRoot {
        schema_version: 1,
        kind,
    }
}

// This bootstrap has no attached adapters. Holding versioned resources alone
// cannot enable F/J or the interpretation stages. Rehearsal G/C stay identity.
fn disconnected(environment: &VersionedEnvironmentV2) -> InitialRuntimeRoot {
    InitialRuntimeRoot {
        schema_version: 1,
        environment: environment.clone(),
        irl: None,
        appraiser: None,
        matcher: None,
        fast: None,
        growth: None,
        dynamics: None,
        compiler: None,
    }
}

fn store_id(objects: &Objects) -> Result<&str> {
    // W02 already binds this canonical location to the writer and its receipt namespace.
    objects
        .root
        .parent()
        .and_then(|p| p.to_str())
        .ok_or(Error::Rejected("invalid store identity"))
}

// Validate both at publication and when replaying the first commit. This runtime
// consumes the existing V2 x/theta pair; D07/D09 snapshots require P5-75.
fn anchor(objects: &Objects, p: &InitialPublication) -> Result<u64> {
    require(
        p.x.object_type == "echo.fast-state-v2",
        "unsupported rehearsal fast-state version",
    )?;
    let x = FastStateSnapshotV2::from_canonical(&objects.read(&p.x)?, objects.config.encoding)?;
    let r = RSnapshotV3::from_canonical(&objects.read(&p.r)?, objects.config.encoding)?;
    for t in &r.tendencies {
        core_types::semantic::check_tendency_v3(t)?;
    }
    // Canonical order by tendency_id on both sets; v2.0 section 6 indexes x by exactly R+.
    require(
        x.learned.iter().map(|v| &v.tendency_id).eq(r
            .tendencies
            .iter()
            .filter(|t| t.pi > 0.5)
            .map(|t| &t.tendency_id)),
        "initial x coordinates differ from R+",
    )?;
    let theta = crate::theta::writable(objects, &p.theta)?;
    require(
        r.law_version == theta.law_version
            && p.q
                .environment
                .law_version
                .as_ref()
                .is_none_or(|law| *law == theta.law_version),
        "initial snapshots bind different laws",
    )?;
    Ok(x.anchor_time_ns.0)
}

pub(crate) fn decode(objects: &Objects, reference: &ObjectRef) -> Result<InitialPublication> {
    require(
        reference.object_type == TAG && reference.resource_version == "1",
        "wrong initial manifest type or version",
    )?;
    Ok(InitialPublication::from_canonical(
        &objects.read(reference)?,
        objects.config.encoding,
    )?)
}

pub(crate) fn verify(
    objects: &Objects,
    reference: &ObjectRef,
    header: &LineageHeaderV2,
    branch: &str,
) -> Result<u64> {
    let p = decode(objects, reference)?;
    require(
        header.lineage == Lineage::REHEARSAL,
        "expected rehearsal initial root",
    )?;
    check_initial_store_binding(
        &p,
        store_id(objects)?,
        branch,
        header,
        &disconnected(&p.q.environment),
        objects.config.encoding,
    )?;
    anchor(objects, &p)
}

// Every derived row can be rebuilt from the exact manifest and the three
// canonical empty prefixes. No cursor or pending D record is consumed.
pub(crate) fn project(
    db: &Connection,
    objects: &Objects,
    reference: &ObjectRef,
    p: &InitialPublication,
) -> Result<()> {
    let id = hex(&reference.digest.0);
    let limits = objects.config.encoding;
    let roles = [
        (
            "H",
            objects.reference(&empty(EmptyPrefixKind::EMPTY_HISTORY), "1")?,
        ),
        ("x", p.x.clone()),
        ("R", p.r.clone()),
        ("theta", p.theta.clone()),
        (
            "L",
            objects.reference(&empty(EmptyPrefixKind::EMPTY_CHANGE_LEDGER), "1")?,
        ),
        ("Q", objects.reference(&p.q, "1")?),
        (
            "U",
            objects.reference(&empty(EmptyPrefixKind::EMPTY_INGRESS_DECISIONS), "1")?,
        ),
    ];
    let mut refs = objects.closure(reference)?;
    for (role, r) in &roles {
        if matches!(*role, "H" | "L" | "Q" | "U") {
            refs.extend(objects.closure(r)?);
        }
        db.execute(
            "INSERT INTO snapshot_objects VALUES(?1,?2,'',?3)",
            params![id, role, r.canonical_bytes(limits)?],
        )?;
    }
    publication::visible(db, objects, &refs)?;
    publication::index_tendencies(db, objects, &id, &p.r)?;
    publication::index_dependencies(db, objects, &id, &refs)?;
    db.execute(
        "INSERT INTO snapshots VALUES(?1,?2)",
        params![id, reference.canonical_bytes(limits)?],
    )?;
    Ok(())
}

impl Store<Rehearsal> {
    /// Bootstrap the local, disconnected rehearsal from explicitly supplied resources.
    /// Preparing objects never exposes them to Reader. The sole writer publishes
    /// all roots together; a lost acknowledgement can be retried with the same inputs.
    pub fn publish_rehearsal_initial(
        &mut self,
        cap: &CommitCapability<Rehearsal>,
        environment: &VersionedEnvironmentV2,
        x: &PreparedObject<Rehearsal>,
        r: &PreparedObject<Rehearsal>,
        theta: &PreparedObject<Rehearsal>,
    ) -> Result<ObjectRef> {
        require(
            Arc::ptr_eq(&cap.owner, &self.objects.pins)
                && [x, r, theta]
                    .iter()
                    .all(|p| Arc::ptr_eq(&p.pins, &self.objects.pins)),
            "foreign initial publication capability",
        )?;
        let runtime = disconnected(environment);
        let history = empty(EmptyPrefixKind::EMPTY_HISTORY);
        let p = InitialPublication {
            schema_version: 1,
            store_id: store_id(&self.objects)?.to_owned(),
            branch_id: self.branch.clone(),
            header: self.header.clone(),
            x: x.reference().clone(),
            r: r.reference().clone(),
            theta: theta.reference().clone(),
            empty_history: Some(history.clone()),
            birth: None,
            q: runtime.clone(),
        };
        let tx = self
            .db
            .transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        require(
            crate::migration::format(&tx)? == 3,
            "initial publication requires format 3",
        )?;
        crate::witness::admit(&tx, &self.root, &self.objects, false)?;
        crate::witness::reconcile(&tx, &self.root, &self.objects, false, &self.fault)?;
        let temporary = tx.query_row(
            "SELECT temporary FROM metadata WHERE singleton=1",
            [],
            |row| row.get(0),
        )?;
        let (header, branch) = crate::store::check_header::<Rehearsal>(
            &tx,
            &self.root,
            self.objects.config,
            temporary,
        )?;
        crate::recovery::verify(&tx, &self.objects, &header, &branch)?;
        let manifest = self.objects.prepare::<Rehearsal, _>(&p, "1", &self.fault)?;
        let reference = manifest.reference();
        let id = hex(&reference.digest.0);
        let _derived = [
            self.objects
                .prepare::<Rehearsal, _>(&history, "1", &self.fault)?,
            self.objects.prepare::<Rehearsal, _>(
                &empty(EmptyPrefixKind::EMPTY_CHANGE_LEDGER),
                "1",
                &self.fault,
            )?,
            self.objects.prepare::<Rehearsal, _>(
                &empty(EmptyPrefixKind::EMPTY_INGRESS_DECISIONS),
                "1",
                &self.fault,
            )?,
            self.objects
                .prepare::<Rehearsal, _>(&runtime, "1", &self.fault)?,
        ];
        let anchor = verify(&self.objects, reference, &header, &branch)?;
        let bytes = reference.canonical_bytes(self.objects.config.encoding)?;
        if crate::store::head(&tx)?.0.is_some() {
            let first: Option<Vec<u8>> = tx
                .query_row(
                    "SELECT snapshot FROM commits WHERE position=0 AND decision='initial'",
                    [],
                    |row| row.get(0),
                )
                .optional()?;
            require(
                first.as_ref() == Some(&bytes),
                "conflicting initial publication",
            )?;
        } else {
            tx.execute(
                "INSERT INTO commits VALUES(?1,NULL,0,?2,?3,'initial')",
                params![id, integer(anchor)?, bytes],
            )?;
            project(&tx, &self.objects, reference, &p)?;
            tx.execute("UPDATE branch_head SET commit_id=?1,processed=0,anchor=?2,snapshot=?3 WHERE singleton=1", params![id, integer(anchor)?, bytes])?;
            self.fault.hit(FaultPoint::BeforePublish)?;
        }
        tx.commit()?;
        self.fault.hit(FaultPoint::SqlCommitted)?;
        crate::witness::record(
            &self.root,
            "commits",
            0,
            reference,
            &self.objects,
            &self.fault,
        )?;
        self.fault.hit(FaultPoint::AfterPublish)?;
        Ok(reference.clone())
    }
}
