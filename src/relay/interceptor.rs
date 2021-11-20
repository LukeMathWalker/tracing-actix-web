use tonic::service::Interceptor;

/// Use with tonic grpc client in actix server context,
/// so that your grpc client automatically pipe the request id to its metadata.
///
/// Here the interceptor doesn't have access to the initial parent request,
/// so it uses a task local to store the request id.
#[derive(Debug, Clone)]
pub struct RelayInterceptor;
impl Interceptor for RelayInterceptor {
    fn call(&mut self, request: tonic::Request<()>) -> Result<tonic::Request<()>, tonic::Status> {
        // safety: must always be set prior first read
        let request_id = super::REQUEST_ID.get();
        let mut request = request;
        request.metadata_mut().insert(
            "request_id",
            request_id
                .to_hyphenated_ref()
                .to_string()
                .parse()
                .expect("Convert request_id to ASCII metadata"),
        );
        Ok(request)
    }
}
