use std::error::Error;

#[derive(Debug)]
pub struct ServiceError {
    pub error: failure::Error,
    pub status_code: http::StatusCode,
}

impl ServiceError {
    pub fn into_http_response(self) -> http::Response<hyper::Body> {
        http::Response::builder()
            .status(self.status_code)
            .body(hyper::Body::from(self.error.to_string()))
            .unwrap()
    }

    pub fn bad_request(err: failure::Error) -> Self {
        ServiceError {
            error: err,
            status_code: http::StatusCode::BAD_REQUEST,
        }
    }

    pub fn unauthorized(err: failure::Error) -> Self {
        ServiceError {
            error: err,
            status_code: http::StatusCode::UNAUTHORIZED,
        }
    }

    pub fn not_found(err: failure::Error) -> Self {
        ServiceError {
            error: err,
            status_code: http::StatusCode::NOT_FOUND,
        }
    }

    pub fn internal_server_error(err: failure::Error) -> Self {
        ServiceError {
            error: err,
            status_code: http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<failure::Error> for ServiceError {
    fn from(err: failure::Error) -> Self {
        ServiceError::unauthorized(err)
    }
}

impl From<debil_mysql::Error> for ServiceError {
    fn from(err: debil_mysql::Error) -> Self {
        use debil_mysql::Error::*;

        match err {
            NotFoundError => ServiceError::not_found(failure::err_msg("record not found")),
            MySQLError(err) => ServiceError::internal_server_error(From::from(err)),
        }
    }
}

impl From<tokio::task::JoinError> for ServiceError {
    fn from(err: tokio::task::JoinError) -> Self {
        ServiceError::internal_server_error(From::from(err))
    }
}

impl<E: Sync + Send + Error + 'static> From<rusoto_core::RusotoError<E>> for ServiceError {
    fn from(err: rusoto_core::RusotoError<E>) -> Self {
        ServiceError::internal_server_error(From::from(err))
    }
}
