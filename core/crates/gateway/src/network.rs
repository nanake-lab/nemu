//! Bounded IO only. No persistence, credential checks or state authority here.
use crate::{
    Error, Result,
    connection::{Authentication, Input},
    framing::{FrameConfig, Framed},
};
use core_types::generated::*;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::{mpsc, oneshot},
};

pub(crate) async fn connection<S>(
    stream: S,
    frames: FrameConfig,
    id: u64,
    actor: mpsc::Sender<Input>,
) where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let _ = run(stream, frames, id, &actor).await;
    let _ = actor.send(Input::Closed { connection: id }).await;
}
async fn run<S: AsyncRead + AsyncWrite + Unpin>(
    stream: S,
    frames: FrameConfig,
    id: u64,
    actor: &mpsc::Sender<Input>,
) -> Result<()> {
    let mut framed = Framed::new(stream, frames)?;
    let request = match framed.read::<TerminalAuthenticate>().await {
        Ok(value) => Authentication(value),
        Err(Error::Disconnected) => return Ok(()),
        Err(_) => {
            let _ = actor.send(Input::Fault { connection: id }).await;
            return Ok(());
        }
    };
    let (output, mut responses) = mpsc::channel(2);
    let (reply, auth_reply) = oneshot::channel();
    actor
        .send(Input::Authenticate {
            connection: id,
            request,
            reply,
            output,
        })
        .await
        .map_err(|_| Error::Disconnected)?;
    let response = auth_reply.await.map_err(|_| Error::Disconnected)?;
    framed.write(&response).await?;
    if response.status != AuthStatus::AUTHENTICATED {
        return Ok(());
    }
    let (read, write) = tokio::io::split(framed.into_inner());
    let mut reader = Framed::new(read, frames)?;
    let mut writer = Framed::new(write, frames)?;
    loop {
        // Keep the SAME read future alive while writing delayed core receipts.
        // Cancelling/restarting a partial frame would desynchronize the stream.
        // No new frame exists until its first byte arrives. Normal waiting for
        // a request/commit receipt is bounded by the actor's session lifetime,
        // not by the partial-frame deadline or an INVALID_FRAME audit record.
        let read = reader.read_after_idle::<TerminalRequest>();
        tokio::pin!(read);
        let request = loop {
            tokio::select! {
                value = &mut read => break value,
                output = responses.recv() => match output {
                    Some(value) => writer.write(&value).await?,
                    None => return Ok(()),
                }
            }
        };
        match request {
            Ok(request) => actor
                .send(Input::Request {
                    connection: id,
                    request: Box::new(request),
                })
                .await
                .map_err(|_| Error::Disconnected)?,
            Err(Error::Disconnected) => return Ok(()),
            Err(_) => {
                let _ = actor.send(Input::Fault { connection: id }).await;
                return Ok(());
            }
        }
    }
}
