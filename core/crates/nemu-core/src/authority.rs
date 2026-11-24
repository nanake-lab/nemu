use core_store::{Rehearsal, Store};
use core_types::generated::IngressReason;
use core_types::{
    contract::Limits,
    generated::{LocalUserBinding, SourceMetadata, TerminalEvent, TerminalSession},
    permission::check_user_text_source,
    scalar::hex,
};
use gateway::{
    Grants,
    hooks::{CoreHandler, Decision},
};
use rusqlite::{Connection, OpenFlags, params};
use sha2::{Digest, Sha256};
use std::path::Path;

pub struct LocalAuthority {
    records: Connection,
    principal: String,
    limits: Limits,
}

impl LocalAuthority {
    pub fn open(path: &Path, sid: &str, limits: Limits) -> Result<Self, rusqlite::Error> {
        let records = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )?;
        records.execute_batch(
            "PRAGMA synchronous=FULL;
            CREATE TABLE IF NOT EXISTS local_auth_v1(
                record_id TEXT PRIMARY KEY, principal_id TEXT NOT NULL,
                installation_id TEXT NOT NULL, session_id TEXT NOT NULL UNIQUE
            ) WITHOUT ROWID;",
        )?;
        let principal = hex(&Sha256::new()
            .chain_update(b"nemu/windows-principal/v1\0")
            .chain_update(sid.as_bytes())
            .finalize());
        Ok(Self {
            records,
            principal,
            limits,
        })
    }

    fn issue(&self, session: &TerminalSession) -> rusqlite::Result<()> {
        // The gateway already chose the 256-bit random session ID. Its record
        // is committed by SQLite before any user event can be authorized.
        self.records.execute(
            "INSERT INTO local_auth_v1 (record_id, principal_id, installation_id, session_id)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                format!("auth-v1-{}", session.session_id),
                self.principal,
                session.installation_id,
                session.session_id
            ],
        )?;
        Ok(())
    }

    fn binding(&self, session: &TerminalSession) -> rusqlite::Result<LocalUserBinding> {
        self.records.query_row(
            "SELECT record_id, principal_id, installation_id, session_id
            FROM local_auth_v1 WHERE session_id=?1 AND principal_id=?2 AND installation_id=?3",
            params![session.session_id, self.principal, session.installation_id],
            |row| {
                Ok(LocalUserBinding {
                    schema_version: 1,
                    authentication_record_id: row.get(0)?,
                    principal_id: row.get(1)?,
                    installation_id: row.get(2)?,
                    session_id: row.get(3)?,
                })
            },
        )
    }
}

impl CoreHandler<Rehearsal> for LocalAuthority {
    fn authenticated(
        &mut self,
        _: &mut Store<Rehearsal>,
        session: &TerminalSession,
    ) -> gateway::Result<Decision<()>> {
        Ok(self.issue(session).map_err(|_| IngressReason::AUTH_FAILED))
    }
    fn authorize_event(
        &mut self,
        _: &Store<Rehearsal>,
        session: &TerminalSession,
        event: &TerminalEvent,
        source: &SourceMetadata,
    ) -> Decision<Grants> {
        let binding = self
            .binding(session)
            .map_err(|_| IngressReason::CAPABILITY_DENIED)?;
        check_user_text_source(session, &binding, event, source, self.limits)
            .map_err(|_| IngressReason::CAPABILITY_DENIED)?;
        Ok(Grants {
            desktop_titles: false,
            codex_sessions: false,
        })
    }
    // The trait denies controls and returns no commit results while waiting for the root.
}
