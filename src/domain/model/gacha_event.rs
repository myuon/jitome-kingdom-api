use crate::domain::model::{GachaEventId, UserId};
use crate::wrapper::unixtime::UnixTime;
use serde::{Serialize, Serializer};

#[derive(Debug, Clone, PartialEq)]
pub enum GachaType {
    Unknown,
    Daily,
}

impl GachaType {
    pub fn to_string(&self) -> String {
        use GachaType::*;

        match self {
            Unknown => "unknown",
            Daily => "daily",
        }
        .to_string()
    }

    pub fn new(rep: &str) -> Self {
        match rep {
            "daily" => GachaType::Daily,
            _ => GachaType::Unknown,
        }
    }
}

impl Serialize for GachaType {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl Default for GachaType {
    fn default() -> Self {
        GachaType::Unknown
    }
}

#[derive(Debug, Clone, Serialize, Default, PartialEq)]
pub struct GachaEvent {
    pub id: GachaEventId,
    pub user_id: UserId,
    pub gacha_type: GachaType,
    pub created_at: UnixTime,
}
