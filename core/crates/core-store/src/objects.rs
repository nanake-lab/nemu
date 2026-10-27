use crate::{Error, Fault, FaultPoint, LineageKind, Result, StoreConfig, error::require, platform};
use core_types::{
    WireMessage,
    contract::{Reader as ByteReader, Schema, append_blob},
    generated::{ObjectRef, RSnapshotV2, RSnapshotV3},
    scalar::{DecimalU64, HexBytes, hex, unhex},
};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    marker::PhantomData,
    path::PathBuf,
    sync::{Arc, Mutex},
};

type Pins = Arc<Mutex<BTreeMap<String, usize>>>;
pub struct PreparedObject<L: LineageKind> {
    pub(crate) reference: ObjectRef,
    pub(crate) closure: Vec<ObjectRef>,
    pub(crate) pins: Pins,
    _lineage: PhantomData<L>,
}
impl<L: LineageKind> PreparedObject<L> {
    pub fn reference(&self) -> &ObjectRef {
        &self.reference
    }
}
impl<L: LineageKind> Drop for PreparedObject<L> {
    fn drop(&mut self) {
        if let Ok(mut pins) = self.pins.lock() {
            for r in &self.closure {
                let key = hex(&r.digest.0);
                if let Some(n) = pins.get_mut(&key) {
                    *n -= 1;
                    if *n == 0 {
                        pins.remove(&key);
                    }
                }
            }
        }
    }
}

pub(crate) struct Objects {
    pub root: PathBuf,
    pub config: StoreConfig,
    pub pins: Pins,
}
impl Objects {
    pub fn new(root: PathBuf, config: StoreConfig) -> Self {
        Self {
            root,
            config,
            pins: Arc::default(),
        }
    }
    pub fn reference<T: WireMessage>(&self, value: &T, version: &str) -> Result<ObjectRef> {
        let schema = Schema::builtin()?;
        let v = serde_json::to_value(value)?;
        Ok(ObjectRef {
            schema_version: 1,
            digest: HexBytes(unhex(&schema.digest(T::NAME, &v, self.config.encoding)?)?),
            object_type: schema.message(T::NAME)?.tag.clone(),
            resource_version: version.into(),
            byte_length: DecimalU64(value.canonical_bytes(self.config.encoding)?.len() as u64),
        })
    }
    pub fn path(&self, r: &ObjectRef) -> Result<PathBuf> {
        r.canonical_bytes(self.config.encoding)?;
        Ok(self.root.join(format!("{}.obj", hex(&r.digest.0))))
    }
    pub fn prepare<L: LineageKind, T: WireMessage>(
        &self,
        value: &T,
        version: &str,
        fault: &Fault,
    ) -> Result<PreparedObject<L>> {
        require(
            !T::SUPERSEDED,
            "new writes must use the active successor message",
        )?;
        let r = self.reference(value, version)?;
        let bytes = value.canonical_bytes(self.config.encoding)?;
        let mut held = self
            .pins
            .lock()
            .map_err(|_| Error::Rejected("object retention lock poisoned"))?;
        self.write(&r, &bytes, fault)?;
        let closure = self.closure(&r)?;
        for item in &closure {
            *held.entry(hex(&item.digest.0)).or_default() += 1;
        }
        Ok(PreparedObject {
            reference: r,
            closure,
            pins: Arc::clone(&self.pins),
            _lineage: PhantomData,
        })
    }
    /// Physical segmentation only: reassembly is byte-identical to generated CanonicalV1.
    /// The index contains the canonical prefix and generated ObjectRef values, never a
    /// second R schema. Unchanged TendencyV3 files are reused by digest.
    pub fn prepare_r<L: LineageKind>(
        &self,
        value: &RSnapshotV3,
        fault: &Fault,
    ) -> Result<PreparedObject<L>> {
        let logical = value.canonical_bytes(self.config.encoding)?;
        let canonical = RSnapshotV3::from_canonical(&logical, self.config.encoding)?;
        let r = self.reference(&canonical, &canonical.revision)?;
        let mut empty = canonical.clone();
        empty.tendencies.clear();
        let mut prefix = empty.canonical_bytes(self.config.encoding)?;
        require(
            prefix.ends_with(&[0, 0, 0, 0]),
            "unexpected R canonical layout",
        )?;
        prefix.truncate(prefix.len() - 4);
        prefix.extend(
            u32::try_from(canonical.tendencies.len())
                .map_err(|_| Error::Rejected("R count overflow"))?
                .to_le_bytes(),
        );
        let mut disk = b"NSI1".to_vec();
        append_blob(&mut disk, &prefix)?;
        let mut reconstructed = prefix;
        let mut held = self
            .pins
            .lock()
            .map_err(|_| Error::Rejected("object retention lock poisoned"))?;
        for tendency in &canonical.tendencies {
            let child = self.reference(tendency, "1")?;
            let bytes = tendency.canonical_bytes(self.config.encoding)?;
            self.write(&child, &bytes, fault)?;
            append_blob(&mut reconstructed, &bytes)?;
            append_blob(&mut disk, &child.canonical_bytes(self.config.encoding)?)?;
        }
        require(
            reconstructed == logical,
            "R physical index differs from canonical encoding",
        )?;
        self.write(&r, &disk, fault)?;
        let closure = self.closure(&r)?;
        for item in &closure {
            *held.entry(hex(&item.digest.0)).or_default() += 1;
        }
        Ok(PreparedObject {
            reference: r,
            closure,
            pins: Arc::clone(&self.pins),
            _lineage: PhantomData,
        })
    }
    fn write(&self, r: &ObjectRef, disk: &[u8], fault: &Fault) -> Result<()> {
        require(
            r.byte_length.0 <= self.config.max_object_bytes,
            "object capacity exhausted",
        )?;
        let path = self.path(r)?;
        if path.exists() {
            self.read(r)?;
            // A previous writer can have died after rename but before directory
            // sync. Reuse must establish durability again before any DB reference.
            fault.hit_object(FaultPoint::ReuseFileSyncBefore, Some(&r.object_type))?;
            fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(&path)?
                .sync_all()?;
            fault.hit_object(FaultPoint::ReuseFileSynced, Some(&r.object_type))?;
            fault.hit_object(FaultPoint::ReuseDirectorySyncBefore, Some(&r.object_type))?;
            platform::sync_dir(&self.root)?;
            fault.hit_object(FaultPoint::ReuseDirectorySynced, Some(&r.object_type))?;
            return Ok(());
        }
        let used = fs::read_dir(&self.root)?.try_fold(0u64, |n, entry| -> Result<u64> {
            n.checked_add(entry?.metadata()?.len())
                .ok_or(Error::Rejected("object size overflow"))
        })?;
        require(
            used.checked_add(disk.len() as u64)
                .is_some_and(|n| n <= self.config.max_store_bytes),
            "object store capacity exhausted",
        )?;
        let mut temp = tempfile::NamedTempFile::new_in(&self.root)?;
        temp.write_all(disk)?;
        fault.hit_object(FaultPoint::ObjectWritten, Some(&r.object_type))?;
        // Check bytes read back before making their name visible.
        require(
            fs::read(temp.path())? == disk,
            "object write verification failed",
        )?;
        temp.as_file().sync_all()?;
        fault.hit_object(FaultPoint::FileSynced, Some(&r.object_type))?;
        platform::persist_noclobber(temp, &path)?;
        fault.hit_object(FaultPoint::Renamed, Some(&r.object_type))?;
        platform::sync_dir(&self.root)?;
        fault.hit_object(FaultPoint::DirectorySynced, Some(&r.object_type))?;
        self.read(r)?;
        Ok(())
    }
    pub fn read(&self, r: &ObjectRef) -> Result<Vec<u8>> {
        self.read_depth(r, 0)
    }
    fn read_depth(&self, r: &ObjectRef, depth: usize) -> Result<Vec<u8>> {
        require(
            depth <= self.config.encoding.max_depth,
            "object depth exhausted",
        )?;
        require(
            r.byte_length.0 <= self.config.max_object_bytes,
            "object too large",
        )?;
        let path = self.path(r)?;
        platform::no_links(&path)?;
        require(
            fs::metadata(&path)?.len()
                <= self
                    .config
                    .max_object_bytes
                    .saturating_add(self.config.encoding.max_bytes as u64),
            "physical object too large",
        )?;
        let disk = fs::read(path)?;
        let bytes = if disk.starts_with(b"NSI1") {
            let schema = Schema::builtin()?;
            require(
                r.object_type == schema.message(RSnapshotV2::NAME)?.tag
                    || r.object_type == schema.message(RSnapshotV3::NAME)?.tag,
                "index has wrong logical type",
            )?;
            let mut input = ByteReader::new(&disk[4..], self.config.encoding)?;
            let mut bytes = input.blob()?.to_vec();
            while !input.done() {
                let child = ObjectRef::from_canonical(input.blob()?, self.config.encoding)?;
                let part = self.read_depth(&child, depth + 1)?;
                append_blob(&mut bytes, &part)?;
                require(
                    bytes.len() <= self.config.encoding.max_bytes,
                    "reconstructed object too large",
                )?;
            }
            bytes
        } else {
            disk
        };
        require(
            bytes.len() as u64 == r.byte_length.0,
            "object length mismatch",
        )?;
        let schema = Schema::builtin()?;
        let (name, _) = schema
            .messages
            .iter()
            .find(|(_, m)| m.tag == r.object_type)
            .ok_or(Error::Rejected("unknown object type"))?;
        let value = schema.decode_canonical(name, &bytes, self.config.encoding, 0)?;
        require(
            schema.digest(name, &value, self.config.encoding)? == hex(&r.digest.0),
            "object digest mismatch",
        )?;
        Ok(bytes)
    }
    pub fn closure(&self, root: &ObjectRef) -> Result<Vec<ObjectRef>> {
        let schema = Schema::builtin()?;
        let mut pending = vec![root.clone()];
        let mut found = BTreeMap::new();
        while let Some(r) = pending.pop() {
            let key = hex(&r.digest.0);
            if let Some(old) = found.get(&key) {
                require(old == &r, "ambiguous object resource version")?;
                continue;
            }
            require(
                found.len() < self.config.encoding.max_items,
                "reference closure capacity exhausted",
            )?;
            let bytes = self.read(&r)?;
            let (name, _) = schema
                .messages
                .iter()
                .find(|(_, m)| m.tag == r.object_type)
                .ok_or(Error::Rejected("unknown object type"))?;
            let value = schema.decode_canonical(name, &bytes, self.config.encoding, 0)?;
            collect_refs(&schema, name, &value, &mut pending)?;
            let disk = fs::read(self.path(&r)?)?;
            if disk.starts_with(b"NSI1") {
                let mut input = ByteReader::new(&disk[4..], self.config.encoding)?;
                input.blob()?;
                while !input.done() {
                    pending.push(ObjectRef::from_canonical(
                        input.blob()?,
                        self.config.encoding,
                    )?);
                }
            }
            found.insert(key, r);
        }
        Ok(found.into_values().collect())
    }
}

pub(crate) fn collect_refs(
    schema: &Schema,
    name: &str,
    value: &Value,
    out: &mut Vec<ObjectRef>,
) -> Result<()> {
    if name == ObjectRef::NAME {
        out.push(serde_json::from_value(value.clone())?);
        return Ok(());
    }
    for field in &schema.message(name)?.fields {
        if schema.messages.contains_key(&field.r#type)
            && let Some(v) = value.get(&field.name)
        {
            if let Some(values) = v.as_array() {
                for v in values {
                    collect_refs(schema, &field.r#type, v, out)?;
                }
            } else {
                collect_refs(schema, &field.r#type, v, out)?;
            }
        }
    }
    Ok(())
}
