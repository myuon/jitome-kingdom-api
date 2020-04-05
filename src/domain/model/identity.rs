use serde::*;

#[derive(Clone, Serialize, Deserialize, Debug, Default, PartialOrd, PartialEq)]
pub struct UserId(pub String);

impl UserId {
    pub fn new() -> Self {
        UserId(uuid::Uuid::new_v4().to_string())
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default, PartialOrd, PartialEq)]
pub struct GachaEventId(pub String);

impl GachaEventId {
    pub fn new() -> Self {
        GachaEventId(uuid::Uuid::new_v4().to_string())
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default, PartialOrd, PartialEq)]
pub struct GiftId(pub String);

impl GiftId {
    pub fn new() -> Self {
        GiftId(uuid::Uuid::new_v4().to_string())
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, Default, PartialOrd, PartialEq)]
pub struct JankenEventId(pub String);

impl JankenEventId {
    pub fn new() -> Self {
        JankenEventId(uuid::Uuid::new_v4().to_string())
    }
}
