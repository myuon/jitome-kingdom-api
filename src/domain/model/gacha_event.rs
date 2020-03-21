use crate::domain::model::UserId;
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
    pub user_id: UserId,
    pub gacha_type: GachaType,
    pub created_at: UnixTime,
}
