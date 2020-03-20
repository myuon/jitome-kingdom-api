use crate::domain::model::UserId;
use crate::wrapper::error::ServiceError;
use serde::*;

#[derive(Serialize, Deserialize, Clone)]
pub struct AuthUser {
    pub user_id: UserId,
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
