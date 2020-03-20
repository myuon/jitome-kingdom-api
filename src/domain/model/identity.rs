use serde::*;

// auth0の発行するsubjectを使う
#[derive(Clone, Serialize, Deserialize)]
pub struct UserId(pub String);
