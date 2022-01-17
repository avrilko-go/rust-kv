use tokio::io::{AsyncRead, AsyncWrite};
use crate::{CommandRequest, CommandResponse, Service, Storage};
use crate::stream::ProstStream;

pub mod stream;
pub mod frame;

pub struct ProstServerStream<S, Store> {
    inner: ProstStream<S, CommandRequest, CommandResponse>,
    service: Service<Store>,
}

pub struct ProstClientStream<S> {
    inner: ProstStream<S, CommandRequest, CommandResponse>,
}

impl<S, Store> ProstServerStream<S, Store>
    where
        S: AsyncRead + AsyncWrite + Send + Unpin + 'static,
        Store: Storage
{
    pub fn new(stream: S, service: Service<Store>) -> Self {
        Self {
            inner: ProstStream::new(stream),
            service,
        }
    }
}