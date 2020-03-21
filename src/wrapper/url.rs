use serde::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Url(pub String);
