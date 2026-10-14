use openfab::oidc::OidcClaims;
use rocket_dyn_templates::tera::Context;
use rocket_oidc::{OIDCGuard, auth::ApiKeyGuard};
use rocket::http::{ContentType, Status};
use rocket::Request;
use std::pin::Pin;
use std::task::{Context as TaskContext, Poll};
use tokio_util::bytes::Bytes;
use rocket::response::{Responder, Response};
use futures_util::{Stream, StreamExt};
use std::io;
use tokio::io::{AsyncRead, ReadBuf};

pub type ApiGuard = ApiKeyGuard<OidcClaims>;
pub type Guard = OIDCGuard<crate::accounts::UserID>;

pub fn global_context() -> Context {
    Context::new()
}

use argon2::{
    Argon2,
    password_hash::{PasswordHasher, PasswordVerifier, phc::PasswordHash},
};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let argon2 = Argon2::default();

    let hash = argon2.hash_password(password.as_bytes())?;

    Ok(hash.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(hash)?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

pub fn unix_epoch_seconds() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is before Unix epoch")
        .as_secs() as i64
}

pub struct ResultStream {
    stream: Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>,
    current_chunk: Option<Bytes>,
    content_type: String,
}

impl ResultStream {
    pub fn new<S>(stream: S, content_type: String) -> Self
    where
        S: Stream<Item = Result<Bytes, reqwest::Error>> + Send + 'static,
    {
        Self {
            stream: Box::pin(stream),
            current_chunk: None,
            content_type,
        }
    }
}

impl AsyncRead for ResultStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut TaskContext<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        loop {
            // 1. If we have leftover data in the current chunk, feed it into the ReadBuf
            if let Some(ref mut chunk) = self.current_chunk {
                if chunk.is_empty() {
                    self.current_chunk = None;
                } else {
                    let len = std::cmp::min(chunk.len(), buf.remaining());
                    buf.put_slice(&chunk[..len]);
                    let _ = chunk.split_to(len); // Advances the slice view
                    if chunk.is_empty() {
                        self.current_chunk = None;
                    }
                    return Poll::Ready(Ok(()));
                }
            }

            // 2. Otherwise, poll the underlying stream for the next chunk
            match self.stream.as_mut().poll_next(cx) {
                Poll::Ready(Some(Ok(chunk))) => {
                    if chunk.is_empty() {
                        continue; // Skip empty chunks and loop again
                    }
                    self.current_chunk = Some(chunk);
                }
                Poll::Ready(Some(Err(e))) => {
                    // Convert stream error into a standard I/O error
                    return Poll::Ready(Err(io::Error::other(e)));
                }
                Poll::Ready(None) => {
                    // EOF reached successfully
                    return Poll::Ready(Ok(()));
                }
                Poll::Pending => {
                    return Poll::Pending;
                }
            }
        }
    }
}

// 2. Implement Rocket's Responder trait so it formats itself into an HTTP response automatically
impl<'r> Responder<'r, 'static> for ResultStream {
    fn respond_to(self, _: &'r Request<'_>) -> Result<Response<'static>, Status> {
        let mut response = Response::new();

        // Parse and set the Content-Type header stored inside the stream object
        let content_type = ContentType::parse_flexible(&self.content_type)
            .unwrap_or(ContentType::Binary);
        response.set_header(content_type);

        // Pass `self` as the streamed body since ResultStream implements AsyncRead
        response.set_streamed_body(self);

        Ok(response)
    }
}