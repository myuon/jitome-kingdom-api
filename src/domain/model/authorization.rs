use crate::wrapper::error::ServiceError;

pub struct Authorization {}

impl Authorization {
    pub fn require_auth(&self) -> Result<(), ServiceError> {
        Ok(())
    }
}
