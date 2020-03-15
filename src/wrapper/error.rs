#[derive(Debug)]
pub struct ServiceError {
    error: failure::Error,
    status_code: http::StatusCode,
}

impl ServiceError {
    pub fn to_http_response(self) -> http::Response<hyper::Body> {
        http::Response::builder()
            .status(self.status_code)
            .body(hyper::Body::from(self.error.to_string()))
            .unwrap()
    }
}
