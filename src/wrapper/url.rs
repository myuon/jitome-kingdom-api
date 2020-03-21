use serde::*;

#[derive(Serialize, Deserialize, Clone)]
pub struct Url(pub String);
