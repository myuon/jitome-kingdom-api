use crate::domain::model::{GachaEventId, UserId};
use crate::wrapper::unixtime::UnixTime;

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct GachaEvent {
    pub id: GachaEventId,
    pub user_id: UserId,
    pub gacha_type: GachaType,
    pub created_at: UnixTime,
}
