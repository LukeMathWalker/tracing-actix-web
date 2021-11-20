use std::{future::Future, pin::Pin};

use crate::RequestId;
use actix_web::{
    body::MessageBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};

/// Use in server actix context,
/// to expose the request id to the [`RelayInterceptor`].
///
/// This middleware is meant to be used in conjunction with [`TracingLogger`] middleware.
#[derive(Clone)]
pub struct Relay;

pub struct RelayMiddleware<S> {
    service: S,
}
impl<S, B> Transform<S, ServiceRequest> for Relay
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Transform = RelayMiddleware<S>;
    type InitError = ();
    type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        std::future::ready(Ok(RelayMiddleware { service }))
    }
}
impl<S, B> Service<ServiceRequest> for RelayMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    actix_web::dev::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let request_id = req.extensions().get::<RequestId>().cloned().unwrap();
        let future =
            super::REQUEST_ID.sync_scope(request_id.clone(), move || self.service.call(req));
        Box::pin(super::REQUEST_ID.scope(request_id, async move {
            let response: Result<Self::Response, Self::Error> = future.await;
            response
        }))
    }
}
