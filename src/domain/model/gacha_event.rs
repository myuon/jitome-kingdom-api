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

#[test]
fn gacha_event_is_available_at() {
    let ev = GachaEvent {
        id: GachaEventId::new(),
        user_id: UserId::new(),
        gacha_type: GachaType::Daily,
        created_at: UnixTime(0),
    };

    assert!(!ev.is_available_at(UnixTime(1)));
    assert!(ev.is_available_at(UnixTime(89400)));
}
