use crate::root_span;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::Error;
use tracing::Span;

pub trait RootSpanBuilder {
    fn on_request_start(request: &ServiceRequest) -> Span;
    fn on_request_end<B>(span: Span, response: &Result<ServiceResponse<B>, Error>);
}

pub struct DefaultRootSpanBuilder;

impl RootSpanBuilder for DefaultRootSpanBuilder {
    fn on_request_start(request: &ServiceRequest) -> Span {
        root_span!(request)
    }

    fn on_request_end<B>(span: Span, response: &Result<ServiceResponse<B>, Error>) {
        match &response {
            Ok(response) => {
                span.record("http.status_code", &response.response().status().as_u16());
                span.record("otel.status_code", &"ok");
            }
            Err(error) => {
                let response_error = error.as_response_error();
                let status_code = response_error.status_code();
                span.record("http.status_code", &status_code.as_u16());

                if status_code.is_client_error() {
                    span.record("otel.status_code", &"ok");
                } else {
                    span.record("otel.status_code", &"error");
                }
            }
        };
    }
}
