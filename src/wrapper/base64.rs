use crate::wrapper::error::ServiceError;
use serde::*;

#[derive(Clone, Serialize, Deserialize)]
pub struct Base64(pub String);

impl Base64 {
    pub fn encode(data: Vec<u8>) -> Self {
        Base64(base64::encode(data))
    }

    pub fn decode(self) -> Result<Vec<u8>, ServiceError> {
        base64::decode(self.0).map_err(|err| {
            ServiceError::bad_request(failure::Error::from_boxed_compat(Box::new(err)))
        })
    }
}
