//! `tracing-actix-web` provides [`TracingLogger`], a middleware to log request and response info
//! when using the [`actix-web`] framework.
//!
//! [`TracingLogger`] is designed as a drop-in replacement of [`actix-web`]'s [`Logger`].
//!
//! [`Logger`] is built on top of the [`log`] crate: you need to use regular expressions to parse
//! the request information out of the logged message.
//!
//! [`TracingLogger`] relies on [`tracing`], a modern instrumentation framework for structured
//! logging: all request information is captured as a machine-parsable set of key-value pairs.
//! It also enables propagation of context information to children spans.
//!
//! [`TracingLogger`]: struct.TracingLogger.html
//! [`actix-web`]: https://docs.rs/actix-web
//! [`Logger`]: https://docs.rs/actix-web/3.0.2/actix_web/middleware/struct.Logger.html
//! [`log`]: https://docs.rs/log
//! [`tracing`]: https://docs.rs/tracing
mod middleware;
mod request_id;
mod root_span_builder;

pub use middleware::TracingLogger;
pub use request_id::RequestId;
pub use root_span_builder::{DefaultRootSpan, RootSpanBuilder};

#[doc(hidden)]
pub mod root_span_macro;

#[cfg(feature = "opentelemetry_0_13")]
mod otel;
