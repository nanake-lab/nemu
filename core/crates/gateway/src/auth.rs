use crate::{Error, Result};
use subtle::ConstantTimeEq;
use zeroize::Zeroizing;

/// Secret bytes have neither Debug nor Serialize. Only ciphertext may be saved.
pub struct TerminalToken(Zeroizing<[u8; 32]>);

/// Process-local capability; never deserialized from a peer or logged.
pub struct AuthenticatedSession {
    pub(crate) device: String,
    pub(crate) session: String,
}
impl TerminalToken {
    #[cfg(windows)]
    pub fn create_protected(path: &std::path::Path) -> Result<Self> {
        let mut bytes = Zeroizing::new([0; 32]);
        crate::windows::random(bytes.as_mut())?;
        let token = Self(bytes);
        let protected = crate::windows::protect(token.0.as_ref(), false)?;
        crate::windows::create_secret_file(path, &protected)?;
        Ok(token)
    }
    #[cfg(windows)]
    pub fn load_protected(path: &std::path::Path) -> Result<Self> {
        // Bounds and reparse-point checks precede allocation and DPAPI.
        let encrypted = crate::windows::read_secret_file(path)?;
        let plain = crate::windows::protect(&encrypted, true)?;
        if plain.len() != 32 {
            return Err(Error::Rejected("invalid terminal token length"));
        }
        let mut token = Self(Zeroizing::new([0; 32]));
        token.0.copy_from_slice(&plain);
        Ok(token)
    }
    /// The IDL adapter must obtain candidate from the approved authentication
    /// message. Identity comes from trusted installation configuration,
    /// never fields claimed by an unauthenticated connection.
    pub fn authenticate(
        &self,
        candidate: &[u8],
        device: &str,
        session: &str,
    ) -> Result<AuthenticatedSession> {
        if candidate.len() != 32 || !bool::from(self.0.as_ref().ct_eq(candidate)) {
            return Err(Error::Rejected("terminal authentication failed"));
        }
        if device.is_empty() || session.is_empty() {
            return Err(Error::Rejected("terminal identity missing"));
        }
        Ok(AuthenticatedSession {
            device: device.into(),
            session: session.into(),
        })
    }
    #[cfg(test)]
    pub(crate) fn synthetic() -> Self {
        Self(Zeroizing::new([0xA5; 32]))
    }
    #[cfg(all(test, windows))]
    pub(crate) fn test_bytes(&self) -> &[u8] {
        self.0.as_ref()
    }
}
