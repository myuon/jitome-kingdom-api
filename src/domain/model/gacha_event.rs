use crate::domain::model::{GachaEventId, UserId};
use crate::wrapper::unixtime::UnixTime;
use serde::{Serialize, Serializer};

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

impl Serialize for GachaType {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct GachaEvent {
    pub id: GachaEventId,
    pub user_id: UserId,
    pub gacha_type: GachaType,
    pub created_at: UnixTime,
}

impl GachaEvent {
    pub fn is_available_at(&self, current_time: UnixTime) -> bool {
        use GachaType::*;

        match self.gacha_type {
            Daily => {
                // デイリーガチャなので前回と違うものだったらOK
                self.created_at.datetime_jst().date() != current_time.datetime_jst().date()
            }
            _ => false,
        }
    }
}
