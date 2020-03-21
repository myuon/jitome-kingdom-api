use serde::*;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UserId(pub String);

impl UserId {
    pub fn new() -> Self {
        UserId(uuid::Uuid::new_v4().to_string())
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GachaEventId(pub String);

impl GachaEventId {
    pub fn new() -> Self {
        GachaEventId(uuid::Uuid::new_v4().to_string())
    }
}
