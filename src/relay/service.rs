use futures::future::BoxFuture;
use hyper::{Request, Response};
use std::convert::TryFrom;
use std::ops::{Deref, DerefMut};
use std::task::{Context, Poll};
use tonic::transport::{Body, NamedService};
use tonic::{body::BoxBody, codegen::Service};
use tracing_futures::Instrument;

use crate::RequestId;

/// Use in tonic server context,
/// so that your service automatically extracts the request id from incoming requests.
///
/// Please note that the request id could still be missing,
/// e.g. if an incoming grpc request was made without setting the request id.
#[derive(Debug, Clone)]
pub struct RelayService<S> {
    inner: S,
}

impl<S> RelayService<S> {
    pub fn new(inner: S) -> Self {
        Self { inner }
    }
}

impl<S: NamedService> NamedService for RelayService<S> {
    const NAME: &'static str = S::NAME;
}

impl<S> Deref for RelayService<S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<S> DerefMut for RelayService<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<S> Service<Request<Body>> for RelayService<S>
where
    S: Service<Request<Body>, Response = Response<BoxBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let mut svc = self.inner.clone();

        let request_id = RequestId::try_from(&req) // search for a value (e.g. relayed by another server)
            .unwrap_or_else(|_| RequestId::generate()); // provide a value if not found (e.g. direct incoming request)

        // TODO: modularity + similar implementation with main feature
        let span = tracing::info_span!(
            "Request",
            request_id = %request_id.clone(),
            request_method = %req.method(),
            request_version = %match req.version() {
              hyper::Version::HTTP_09 => "0.9".to_string(),
              hyper::Version::HTTP_10 => "1.0".to_string(),
              hyper::Version::HTTP_11 => "1.1".to_string(),
              hyper::Version::HTTP_2 => "2.0".to_string(),
              hyper::Version::HTTP_3 => "3.0".to_string(),
              other => format!("{:?}", other),
            },
            request_uri = %req.uri(),
        );
        // safety: must always be set prior first read
        let fut = super::REQUEST_ID.sync_scope(request_id.clone(), move || svc.call(req));
        Box::pin(
            super::REQUEST_ID
                .scope(request_id, async move { fut.await })
                .instrument(span),
        )
    }
}
