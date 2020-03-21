use serde::*;

// auth0の発行するsubjectを使う
#[derive(Clone, Serialize, Deserialize)]
pub struct UserId(pub String);

impl UserId {
    pub fn new() -> Self {
        UserId(uuid::Uuid::new_v4().to_string())
    }
}
