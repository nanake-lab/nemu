//! Bounded u32 little-endian length prefix + generated Protobuf message.
//! A timed-out/failed operation poisons this connection: a partial frame must
//! never be retried on the same stream. This is not an RPC dispatch protocol.
use crate::{Error, Result};
use core_types::{WireMessage, contract::Limits};
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use zeroize::Zeroizing;

#[derive(Clone, Copy)]
pub struct FrameConfig {
    pub encoding: Limits,
    pub deadline: Duration,
}
impl FrameConfig {
    pub(crate) fn validate(self) -> Result<()> {
        if self.encoding.max_bytes == 0
            || self.encoding.max_bytes > u32::MAX as usize
            || self.encoding.max_items == 0
            || self.encoding.max_depth == 0
            || self.deadline.is_zero()
        {
            return Err(Error::Rejected("invalid frame limits"));
        }
        Ok(())
    }
}
pub struct Framed<S> {
    stream: S,
    config: FrameConfig,
    failed: bool,
}
impl<S> Framed<S> {
    pub fn new(stream: S, config: FrameConfig) -> Result<Self> {
        config.validate()?;
        Ok(Self {
            stream,
            config,
            failed: false,
        })
    }
    #[cfg(windows)]
    pub(crate) fn into_inner(self) -> S {
        self.stream
    }
}
impl<S: AsyncRead + Unpin> Framed<S> {
    pub async fn read<T: WireMessage>(&mut self) -> Result<T> {
        if self.failed {
            return Err(Error::Rejected("connection is closed"));
        }
        self.failed = true; // Cancellation by the caller is also fail-closed.
        // Historical decoding belongs to core-types, never a live terminal.
        if T::SUPERSEDED {
            return Err(Error::Rejected("superseded inbound message"));
        }
        let limits = self.config.encoding;
        let result = tokio::time::timeout(self.config.deadline, async {
            let mut header = [0; 4];
            if self.stream.read(&mut header[..1]).await? == 0 {
                return Err(Error::Disconnected);
            }
            self.stream.read_exact(&mut header[1..]).await?;
            let size = u32::from_le_bytes(header) as usize;
            if size == 0 || size > limits.max_bytes {
                return Err(Error::Rejected("frame size exceeded"));
            }
            let mut bytes = Zeroizing::new(vec![0; size]);
            self.stream.read_exact(&mut bytes).await?;
            T::from_protobuf(&bytes, limits).map_err(|_| Error::Contract)
        })
        .await
        .map_err(|_| Error::Timeout)?;
        if result.is_ok() {
            self.failed = false;
        }
        result
    }
}
impl<S: AsyncWrite + Unpin> Framed<S> {
    pub async fn write<T: WireMessage>(&mut self, value: &T) -> Result<()> {
        if self.failed {
            return Err(Error::Rejected("connection is closed"));
        }
        if T::SUPERSEDED {
            return Err(Error::Rejected("superseded outbound message"));
        }
        let bytes = Zeroizing::new(
            value
                .protobuf_bytes(self.config.encoding)
                .map_err(|_| Error::Contract)?,
        );
        let size = u32::try_from(bytes.len()).map_err(|_| Error::Contract)?;
        if size == 0 {
            return Err(Error::Contract);
        }
        self.failed = true;
        let result = tokio::time::timeout(self.config.deadline, async {
            self.stream.write_all(&size.to_le_bytes()).await?;
            self.stream.write_all(&bytes).await?;
            self.stream.flush().await?;
            Ok(())
        })
        .await
        .map_err(|_| Error::Timeout)?;
        if result.is_ok() {
            self.failed = false;
        }
        result
    }
}
