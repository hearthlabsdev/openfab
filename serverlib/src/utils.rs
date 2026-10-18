use crate::errors::FileMatchErr;
use futures_util::Stream;
use rand::Rng;
use rand::distr::Alphanumeric;
use rocket::Request;
use rocket::http::{ContentType, Status};
use rocket::response::{Responder, Response};
use rocket_oidc::OIDCGuard;
use std::io;
use std::pin::Pin;
use std::task::{Context as TaskContext, Poll};
use tokio::io::{AsyncRead, ReadBuf};
use tokio_util::bytes::Bytes;

//pub type ApiGuard = ApiKeyGuard<crate::accounts::UserID>;

pub type Guard = OIDCGuard<crate::accounts::UserID>;

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
        let content_type =
            ContentType::parse_flexible(&self.content_type).unwrap_or(ContentType::Binary);
        response.set_header(content_type);

        // Pass `self` as the streamed body since ResultStream implements AsyncRead
        response.set_streamed_body(self);

        Ok(response)
    }
}

pub fn random_string(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

/*
// a way to abstract pagination, things to consider: binding, and ordering.
#[async_trait::async_trait]
pub trait Pagination: ormlite::Model {
    fn paginate(pool: &PgPool, page: Option<i64>) -> Result<Vec<Self>, OpenFabErr> {
        Ok(Self::select()
            .limit(PAGE_SIZE)
            .offset(page)
            .fetch_all(&mut *conn)
            .await?)
    }
}*/

pub trait FileID {
    fn hash_alg(&self) -> &str;
    fn hash(&self) -> &str;
    fn size(&self) -> i64;
    fn modified(&self) -> i64;
    fn verify(&self, other: &Self) -> Result<(), FileMatchErr> {
        if self.hash_alg() != other.hash_alg() {
            return Err(FileMatchErr::HashAlg(
                self.hash_alg().to_string(),
                other.hash_alg().to_string(),
            ));
        }

        if self.hash() != other.hash() {
            return Err(FileMatchErr::Hash(
                self.hash().to_string(),
                other.hash().to_string(),
            ));
        }

        if self.size() != other.size() {
            return Err(FileMatchErr::Size(self.size(), other.size()));
        }

        if self.modified() != other.modified() {
            return Err(FileMatchErr::Modified(self.modified(), other.modified()));
        }

        Ok(())
    }
}
