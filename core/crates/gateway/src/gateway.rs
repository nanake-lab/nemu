//! Single-writer actor: connection tasks have no Store, token or business access.
pub use crate::downlink::TerminalOutput;
use crate::{
    Error, Result, TerminalToken,
    connection::{Connection, Input, Pending},
    framing::FrameConfig,
    hooks::CoreHandler,
};
use core_store::{LineageKind, Store};
use core_types::{WireMessage, generated::*, terminal_ingress::check_session};
use std::{collections::BTreeMap, time::Duration};

#[cfg(test)]
pub(crate) type VerificationGate = (
    tokio::sync::oneshot::Sender<()>,
    std::sync::mpsc::Receiver<()>,
);

/// Deployment setting in seconds. Read once at startup, outside the event loop.
fn verification_interval(setting: Option<std::ffi::OsString>) -> Result<Duration> {
    let Some(setting) = setting else {
        return Ok(Duration::from_secs(24 * 60 * 60));
    };
    let seconds = setting
        .to_str()
        .and_then(|v| v.parse::<u64>().ok())
        .filter(|&v| v > 0)
        .ok_or(Error::Rejected(
            "NEMU_STORE_VERIFY_INTERVAL_SECONDS must be a positive integer",
        ))?;
    Ok(Duration::from_secs(seconds))
}

pub struct Installation {
    pub(crate) id: String,
    pub(crate) token: TerminalToken,
    pub(crate) capabilities: Vec<IngressKind>,
}
impl Installation {
    pub fn new(id: String, token: TerminalToken, capabilities: Vec<IngressKind>) -> Self {
        Self {
            id,
            token,
            capabilities,
        }
    }
}
#[derive(Clone)]
pub struct GatewayConfig {
    /// Hard limit applies BEFORE authentication; never sourced from peer fields.
    pub frames: FrameConfig,
    pub budget: TerminalBudget,
    pub audit: IngressAuditBudget,
    pub max_connections: usize,
    pub max_pending_requests: usize,
    pub poll_interval: Duration,
    pub upload_lifetime: Duration,
    pub session_lifetime: Duration,
}
pub struct Gateway<L: LineageKind, H: CoreHandler<L>> {
    pub(crate) state: Option<RunStateV2>,
    output_tx: tokio::sync::mpsc::Sender<(String, TerminalOutput)>,
    output_rx: tokio::sync::mpsc::Receiver<(String, TerminalOutput)>,
    pub(crate) store: Store<L>,
    pub(crate) handler: H,
    pub(crate) config: GatewayConfig,
    pub(crate) installations: Vec<Installation>,
    pub(crate) connections: BTreeMap<u64, Connection<L>>,
    pub(crate) pending: BTreeMap<u64, Pending>,
    pub(crate) receipt_reader: Option<core_store::Reader<L>>,
    #[cfg(test)]
    pub(crate) check_gate: Option<VerificationGate>,
    #[cfg(test)]
    pub(crate) full_gate: Option<VerificationGate>,
    pub(crate) audit_blocked: bool,
    deferred: std::collections::VecDeque<(u64, Box<TerminalRequest>)>,
    audit_bytes: usize,
    verification_interval: Duration,
}
impl<L: LineageKind, H: CoreHandler<L>> Gateway<L, H> {
    pub fn new(
        store: Store<L>,
        handler: H,
        installations: Vec<Installation>,
        config: GatewayConfig,
    ) -> Result<Self> {
        if L::KIND == Lineage::EXPERIMENT {
            return Err(Error::Rejected(
                "experiment lineage has no terminal service",
            ));
        }
        config.frames.validate()?;
        config
            .audit
            .canonical_bytes(config.frames.encoding)
            .map_err(|_| Error::Contract)?;
        if installations.is_empty()
            || installations.len() > config.frames.encoding.max_items
            || config.max_connections == 0
            || config.max_connections > 14
            || config.max_pending_requests == 0
            || config.max_pending_requests > config.frames.encoding.max_items
            || config.poll_interval.is_zero()
            || config.upload_lifetime.is_zero()
            || config.session_lifetime.is_zero()
            || config.budget.max_frame_bytes as usize > config.frames.encoding.max_bytes
            || config.budget.max_object_bytes as usize > config.frames.encoding.max_bytes
        {
            return Err(Error::Rejected("invalid gateway deployment limits"));
        }
        let mut ids = std::collections::BTreeSet::new();
        for i in &installations {
            if i.id.is_empty() || !ids.insert(&i.id) {
                return Err(Error::Rejected("invalid installation configuration"));
            }
            check_session(
                &TerminalSession {
                    schema_version: 1,
                    session_id: "configuration-check".into(),
                    installation_id: i.id.clone(),
                    protocol_version: 1,
                    capabilities: i.capabilities.clone(),
                    budget: config.budget.clone(),
                },
                config.frames.encoding,
            )
            .map_err(|_| Error::Contract)?;
        }
        let records = store.ingress_rejections()?;
        let bytes: usize = records
            .iter()
            .map(|r| r.canonical_bytes(config.frames.encoding).map(|b| b.len()))
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|_| Error::Contract)?
            .iter()
            .sum();
        let audit_blocked = records.len() >= config.audit.max_records as usize
            || bytes >= config.audit.max_bytes as usize;
        let (output_tx, output_rx) = tokio::sync::mpsc::channel(config.max_pending_requests);
        Ok(Self {
            state: None,
            output_tx,
            output_rx,
            store,
            handler,
            installations,
            config,
            connections: BTreeMap::new(),
            pending: BTreeMap::new(),
            receipt_reader: None,
            #[cfg(test)]
            check_gate: None,
            #[cfg(test)]
            full_gate: None,
            audit_blocked,
            deferred: Default::default(),
            audit_bytes: bytes,
            verification_interval: verification_interval(std::env::var_os(
                "NEMU_STORE_VERIFY_INTERVAL_SECONDS",
            ))?,
        })
    }
    /// Monotonic running time between idle full-history checks. Each service
    /// start checks immediately, including when the schedule is reconfigured.
    pub fn set_verification_interval(&mut self, interval: Duration) -> Result<()> {
        if interval.is_zero() {
            return Err(Error::Rejected("zero verification interval"));
        }
        self.verification_interval = interval;
        Ok(())
    }

    /// Return the sole writer to its owner; no shared writer or second journal.
    pub fn into_parts(self) -> (Store<L>, H) {
        (self.store, self.handler)
    }
    pub fn store(&self) -> &Store<L> {
        &self.store
    }
    /// The trusted producer addresses the initiating authenticated session.
    pub fn output_sender(&self) -> tokio::sync::mpsc::Sender<(String, TerminalOutput)> {
        self.output_tx.clone()
    }
    pub fn handler_mut(&mut self) -> &mut H {
        &mut self.handler
    }
    pub(crate) fn defer(&mut self, connection: u64, request: Box<TerminalRequest>) {
        if self.deferred.len() >= self.config.max_pending_requests {
            self.close(connection);
        } else {
            self.deferred.push_back((connection, request));
        }
    }
    pub(crate) fn close(&mut self, connection: u64) {
        self.connections.remove(&connection); // drops all unfinished buffers/pins
        self.pending.retain(|_, p| p.connection != connection);
        self.deferred.retain(|(id, _)| *id != connection);
    }
    pub(crate) fn audit(
        &mut self,
        reason: IngressReason,
        kind: Option<IngressKind>,
    ) -> Result<IngressReason> {
        if self.audit_blocked {
            return Ok(IngressReason::AUDIT_FULL);
        }
        match self
            .store
            .audit_ingress_rejection(reason, kind, &self.config.audit)
        {
            Ok(record) => {
                self.audit_bytes = self.audit_bytes.saturating_add(
                    record
                        .canonical_bytes(self.config.frames.encoding)
                        .map_err(|_| Error::Contract)?
                        .len(),
                );
                if record.position.0 + 1 >= u64::from(self.config.audit.max_records)
                    || self.audit_bytes >= self.config.audit.max_bytes as usize
                {
                    self.audit_blocked = true;
                }
                Ok(reason)
            }
            Err(core_store::Error::Backpressure(IngressReason::AUDIT_FULL)) => {
                self.audit_blocked = true;
                Ok(IngressReason::AUDIT_FULL)
            }
            Err(e) => Err(e.into()), // outcome unknown: no invented receipt
        }
    }
    pub(crate) fn input(&mut self, input: Input) -> Result<()> {
        match input {
            Input::Authenticate {
                connection,
                request,
                reply,
                output,
            } => {
                self.authenticate(connection, request, reply, output)?;
            }
            Input::Request {
                connection,
                request,
            } => self.dispatch(connection, *request)?,
            Input::Closed { connection } => self.close(connection),
            Input::Fault { connection } => {
                self.audit(IngressReason::INVALID_FRAME, None)?;
                self.close(connection);
            }
        }
        Ok(())
    }
    fn expire_connections(&mut self) {
        let expired: Vec<_> = self
            .connections
            .iter()
            .filter(|(_, c)| {
                c.output.is_closed() || c.created.elapsed() >= self.config.session_lifetime
            })
            .map(|(id, _)| *id)
            .collect();
        for id in expired {
            self.close(id);
        }
        for c in self.connections.values_mut() {
            c.uploads.expire(self.config.upload_lifetime);
        }
    }
    /// Bind only on the current user's SID pipe. Multiple I/O tasks submit to
    /// this actor; all Store and handler access stays serial in this future.
    #[cfg(windows)]
    pub async fn serve(&mut self, shutdown: impl std::future::Future<Output = ()>) -> Result<()> {
        let listener = crate::pipe::Listener::bind(&self.store, self.config.frames)?;
        self.serve_listener(listener, shutdown).await
    }
    #[cfg(windows)]
    pub(crate) async fn serve_listener(
        &mut self,
        mut listener: crate::pipe::Listener,
        shutdown: impl std::future::Future<Output = ()>,
    ) -> Result<()> {
        let (tx, mut rx) = tokio::sync::mpsc::channel(self.config.max_connections * 2);
        let mut tasks = tokio::task::JoinSet::new();
        let mut timer = tokio::time::interval(self.config.poll_interval);
        timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut sequence = 0u64;
        let mut checking = Some(self.check_publication(true, None)?);
        self.store.set_cached_reads_only(true);
        let mut last_full = std::time::Instant::now();
        tokio::pin!(shutdown);
        let result = async { loop {
            let step: Result<()> = tokio::select! {
                _ = &mut shutdown => break Ok(()),
                Some((session, message)) = self.output_rx.recv() => self.send_output(&session, message),
                accept = listener.accept_owned() => match accept {
                    Ok(stream) => {
                        if tasks.len() >= self.config.max_connections {
                            self.audit(IngressReason::CAPABILITY_DENIED, None)?;
                            drop(stream);
                        } else {
                            sequence = sequence.checked_add(1).ok_or(Error::Rejected("connection sequence exhausted"))?;
                            tasks.spawn(crate::network::connection(stream.into_inner(), self.config.frames, sequence, self.config.max_pending_requests, tx.clone()));
                        }
                        Ok(())
                    }
                    Err(Error::Timeout) => Ok(()),
                    Err(e) => Err(e),
                },
                input = rx.recv() => match input.ok_or(Error::Disconnected)? {
                    Input::Request { connection, request }
                        if self.deferred.iter().any(|(id, _)| *id == connection) ||
                           (!matches!(request.kind, IngressKind::INGRESS_QUERY | IngressKind::INGRESS_UPLOAD) && !self.store.has_verified_publication()?) => {
                        // Preserve frame order on a waiting pipe. A saturated peer
                        // cannot prevent other connections from authenticating or querying.
                        self.defer(connection, request);
                        if checking.is_none() { checking = Some(self.check_publication(false, None)?); }
                        Ok(())
                    }
                    input => self.input(input),
                },
                completed = async { checking.as_mut().unwrap().await }, if checking.is_some() => {
                    checking = None;
                    self.finish_progress(completed.map_err(|_| Error::Rejected("publication check worker failed"))??)?;
                    let waiting = std::mem::take(&mut self.deferred);
                    for (connection, request) in waiting {
                        if self.connections.contains_key(&connection) {
                            self.dispatch(connection, *request)?;
                        }
                    }
                    Ok(())
                },
                _ = timer.tick() => {
                    self.expire_connections();
                    if checking.is_none() {
                        checking = if !self.store.has_verified_publication()? {
                            Some(self.check_publication(false, None)?)
                        } else if last_full.elapsed() >= self.verification_interval
                            && self.pending.is_empty() && self.store.next_raw()?.is_none() {
                            last_full = std::time::Instant::now();
                            Some(self.check_publication(true, None)?)
                        } else { self.progress()? };
                    }
                    Ok(())
                },
                _ = tasks.join_next(), if !tasks.is_empty() => Ok(()),
            };
            if let Err(e) = step { break Err(e); }
            // Store operations are synchronous. Let native I/O tasks and peers
            // drain bounded replies between steps even when a timer is overdue.
            tokio::task::yield_now().await;
        } }.await;
        // A running blocking job cannot be aborted. Join it before returning
        // the Store to its owner, including on shutdown and error.
        let joined = if let Some(check) = checking {
            check
                .await
                .map_err(|_| Error::Rejected("publication check worker failed"))
                .and_then(|r| r.map(|_| ()))
        } else {
            Ok(())
        };
        self.store.set_cached_reads_only(false);
        tasks.abort_all();
        while tasks.join_next().await.is_some() {}
        self.connections.clear();
        self.pending.clear();
        self.deferred.clear();
        result.and(joined)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verification_schedule_uses_a_positive_user_interval_or_daily_default() {
        assert_eq!(
            verification_interval(None).unwrap(),
            Duration::from_secs(86_400)
        );
        assert_eq!(
            verification_interval(Some("3600".into())).unwrap(),
            Duration::from_secs(3600)
        );
        assert!(verification_interval(Some("0".into())).is_err());
        assert!(verification_interval(Some("invalid".into())).is_err());
    }
}
