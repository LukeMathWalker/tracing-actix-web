#[macro_export]
/// [`root_span!`] creates a new [`tracing::Span`].
/// It empowers you to add custom properties to the root span on top of the HTTP properties tracked
/// by [`DefaultRootSpanBuilder`].
///
/// # Why a macro?
///
/// `tracing` requires all the properties attached to a span to be declared upfront, when the span is created.
/// You cannot add new ones afterwards.
/// This makes it extremely fast, but it pushes us to reach for macros when we need some level of composition.
///
/// # Macro syntax
///
/// The first argument passed to `root_span!` must be a reference to an [`actix_web::dev::ServiceRequest`].
///
/// ```rust
/// use actix_web::body::MessageBody;
/// use actix_web::dev::{ServiceResponse, ServiceRequest};
/// use actix_web::Error;
/// use tracing_actix_web::{TracingLogger, DefaultRootSpanBuilder, RootSpanBuilder, root_span};
/// use tracing::Span;
///
/// pub struct CustomRootSpanBuilder;
///
/// impl RootSpanBuilder for CustomRootSpanBuilder {
///     fn on_request_start(request: &ServiceRequest) -> Span {
///         root_span!(request)
///     }
///
///     fn on_request_end<B: MessageBody>(span: Span, outcome: &Result<ServiceResponse<B>, Error>) {
///         DefaultRootSpanBuilder::on_request_end(span, outcome);
///     }
/// }
/// ```
///
/// If nothing else is specified, the span generated by `root_span!` is identical to the one you'd
/// get by using `DefaultRootSpanBuilder`.
///
/// You can define new fields following the same syntax of `tracing::info_span!` for fields:
///
/// ```rust,should_panic
/// # let request: &actix_web::dev::ServiceRequest = todo!();
/// use tracing_actix_web::Level;
///
/// // Define a `client_id` field as empty. It might be populated later.
/// tracing_actix_web::root_span!(request, client_id = tracing::field::Empty);
///
/// // Define a `name` field with a known value, `AppName`.
/// tracing_actix_web::root_span!(request, name = "AppName");
///
/// // Define an `app_id` field using the variable with the same name as value.
/// let app_id = "XYZ";
/// tracing_actix_web::root_span!(request, app_id);
///
/// // Use a custom level, `DEBUG`, instead of the default (`INFO`).
/// tracing_actix_web::root_span!(level = Level::DEBUG, request);
///
/// // All together
/// tracing_actix_web::root_span!(request, client_id = tracing::field::Empty, name = "AppName", app_id);
/// ```
///
/// [`DefaultRootSpanBuilder`]: crate::DefaultRootSpanBuilder
macro_rules! root_span {
    // Vanilla root span, with no additional fields
    ($request:ident) => {
        $crate::root_span!($request,)
    };
    // Vanilla root span, with a level but no additional fields
    (level = $lvl:expr, $request:ident) => {
        $crate::root_span!(level = $lvl, $request,)
    };
    // One or more additional fields, comma separated, without a level
    ($request:ident, $($field:tt)*) => {
        $crate::root_span!(level = $crate::Level::INFO, $request, $($field)*)
    };
    // One or more additional fields, comma separated
    (level = $lvl:expr, $request:ident, $($field:tt)*) => {
        {
            let user_agent = $request
                .headers()
                .get("User-Agent")
                .map(|h| h.to_str().unwrap_or(""))
                .unwrap_or("");
            let http_route: std::borrow::Cow<'static, str> = $request
                .match_pattern()
                .map(Into::into)
                .unwrap_or_else(|| "default".into());
            let http_method = $crate::root_span_macro::private::http_method_str($request.method());
            let connection_info = $request.connection_info();
            let request_id = $crate::root_span_macro::private::get_request_id($request);

            macro_rules! inner_span {
                ($level:expr) => {
                    $crate::root_span_macro::private::tracing::span!(
                        $level,
                        "HTTP request",
                        http.method = %http_method,
                        http.route = %http_route,
                        http.flavor = %$crate::root_span_macro::private::http_flavor($request.version()),
                        http.scheme = %$crate::root_span_macro::private::http_scheme(connection_info.scheme()),
                        http.host = %connection_info.host(),
                        http.client_ip = %$request.connection_info().realip_remote_addr().unwrap_or(""),
                        http.user_agent = %user_agent,
                        http.target = %$request.uri().path_and_query().map(|p| p.as_str()).unwrap_or(""),
                        http.status_code = $crate::root_span_macro::private::tracing::field::Empty,
                        otel.name = %format!("HTTP {} {}", http_method, http_route),
                        otel.kind = "server",
                        otel.status_code = $crate::root_span_macro::private::tracing::field::Empty,
                        trace_id = $crate::root_span_macro::private::tracing::field::Empty,
                        request_id = %request_id,
                        exception.message = $crate::root_span_macro::private::tracing::field::Empty,
                        // Not proper OpenTelemetry, but their terminology is fairly exception-centric
                        exception.details = $crate::root_span_macro::private::tracing::field::Empty,
                        $($field)*
                    )
                };
            }
            let span = match $lvl {
                $crate::Level::TRACE => inner_span!($crate::Level::TRACE),
                $crate::Level::DEBUG => inner_span!($crate::Level::DEBUG),
                $crate::Level::INFO => inner_span!($crate::Level::INFO),
                $crate::Level::WARN => inner_span!($crate::Level::WARN),
                $crate::Level::ERROR => inner_span!($crate::Level::ERROR),
            };
            std::mem::drop(connection_info);

            // Previously, this line was instrumented with an opentelemetry-specific feature
            // flag check. However, this resulted in the feature flags being resolved in the crate
            // which called `root_span!` as opposed to being resolved by this crate as expected.
            // Therefore, this function simply wraps an internal function with the feature flags
            // to ensure that the flags are resolved against this crate.
            $crate::root_span_macro::private::set_otel_parent(&$request, &span);

            span
        }
    };
}

#[doc(hidden)]
pub mod private {
    //! This module exposes and re-exports various functions and traits as public in order to leverage them
    //! in the code generated by the `root_span` macro.
    //! Items in this module are not part of the public interface of `tracing-actix-web` - they are considered
    //! implementation details and will change without notice in patch, minor and major releases.
    use crate::RequestId;
    use actix_web::dev::ServiceRequest;
    use actix_web::http::{Method, Version};
    use std::borrow::Cow;

    pub use tracing;

    #[doc(hidden)]
    // We need to allow unused variables because the function
    // body is empty if the user of the library chose not to activate
    // any OTEL feature.
    #[allow(unused_variables)]
    pub fn set_otel_parent(req: &ServiceRequest, span: &tracing::Span) {
        #[cfg(any(
            feature = "opentelemetry_0_13",
            feature = "opentelemetry_0_14",
            feature = "opentelemetry_0_15",
            feature = "opentelemetry_0_16",
            feature = "opentelemetry_0_17",
            feature = "opentelemetry_0_18",
            feature = "opentelemetry_0_19",
            feature = "opentelemetry_0_20",
            feature = "opentelemetry_0_21",
            feature = "opentelemetry_0_22",
        ))]
        crate::otel::set_otel_parent(req, span);
    }

    #[doc(hidden)]
    #[inline]
    pub fn http_method_str(method: &Method) -> Cow<'static, str> {
        match method {
            &Method::OPTIONS => "OPTIONS".into(),
            &Method::GET => "GET".into(),
            &Method::POST => "POST".into(),
            &Method::PUT => "PUT".into(),
            &Method::DELETE => "DELETE".into(),
            &Method::HEAD => "HEAD".into(),
            &Method::TRACE => "TRACE".into(),
            &Method::CONNECT => "CONNECT".into(),
            &Method::PATCH => "PATCH".into(),
            other => other.to_string().into(),
        }
    }

    #[doc(hidden)]
    #[inline]
    pub fn http_flavor(version: Version) -> Cow<'static, str> {
        match version {
            Version::HTTP_09 => "0.9".into(),
            Version::HTTP_10 => "1.0".into(),
            Version::HTTP_11 => "1.1".into(),
            Version::HTTP_2 => "2.0".into(),
            Version::HTTP_3 => "3.0".into(),
            other => format!("{other:?}").into(),
        }
    }

    #[doc(hidden)]
    #[inline]
    pub fn http_scheme(scheme: &str) -> Cow<'static, str> {
        match scheme {
            "http" => "http".into(),
            "https" => "https".into(),
            other => other.to_string().into(),
        }
    }

    #[doc(hidden)]
    pub fn generate_request_id() -> RequestId {
        RequestId::generate()
    }

    #[doc(hidden)]
    pub fn get_request_id(request: &ServiceRequest) -> RequestId {
        use actix_web::HttpMessage;

        request.extensions().get::<RequestId>().copied().unwrap()
    }
}
