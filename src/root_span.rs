use actix_web::dev::Payload;
use actix_web::{FromRequest, HttpRequest};
use std::future::{ready, Ready};
use tracing::Span;

#[derive(Clone)]
pub struct RootSpan(Span);

impl RootSpan {
    pub(crate) fn new(span: Span) -> Self {
        Self(span)
    }
}

impl std::ops::Deref for RootSpan {
    type Target = Span;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::convert::Into<Span> for RootSpan {
    fn into(self) -> Span {
        self.0
    }
}

impl FromRequest for RootSpan {
    type Error = ();
    type Future = Ready<Result<Self, Self::Error>>;
    type Config = ();

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        ready(req.extensions().get::<RootSpan>().cloned().ok_or(()))
    }
}
