use crate::wrapper::error::ServiceError;
use serde::*;

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct AuthUser {
    pub subject: String,
}

pub struct Authorization {
    result: Result<AuthUser, ServiceError>,
}

impl Authorization {
    pub fn new(result: Result<AuthUser, ServiceError>) -> Authorization {
        Authorization { result }
    }

    pub fn require_auth(self) -> Result<AuthUser, ServiceError> {
        self.result
    }
}
