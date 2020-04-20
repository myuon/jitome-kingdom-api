use crate::domain::model::{GiftId, JankenEventId};
use crate::wrapper::error::ServiceError;
use crate::wrapper::unixtime::UnixTime;
use serde::*;

#[derive(Clone, Debug, Serialize, Deserialize, PartialOrd, PartialEq)]
pub enum GiftType {
    #[serde(rename = "point")]
    Point(u64),
}

#[derive(Clone, Debug, PartialOrd, PartialEq)]
pub enum GiftStatus {
    Unknown,
    Ready,
    Opened,
}

impl GiftStatus {
    pub fn to_string(&self) -> String {
        use GiftStatus::*;

        match self {
            Unknown => "unknown",
            Ready => "ready",
            Opened => "opened",
        }
        .to_string()
    }

    pub fn from_str(rep: &str) -> Self {
        match rep {
            "ready" => GiftStatus::Ready,
            "opened" => GiftStatus::Opened,
            _ => GiftStatus::Unknown,
        }
    }
}

impl Serialize for GiftStatus {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Gift {
    pub id: GiftId,
    pub gift_type: GiftType,
    pub description: String,
    pub created_at: UnixTime,
    pub status: GiftStatus,
    pub janken_win_event: Option<JankenEventId>,
    pub janken_lose_event: Option<JankenEventId>,
}

impl Gift {
    pub fn new(gift_type: GiftType, description: String) -> Self {
        Gift {
            id: GiftId::new(),
            gift_type,
            description,
            created_at: UnixTime::now(),
            status: GiftStatus::Ready,
            janken_win_event: None,
            janken_lose_event: None,
        }
    }

    pub fn open(&mut self) -> Result<(), ServiceError> {
        if self.status != GiftStatus::Ready {
            return Err(ServiceError::bad_request(failure::err_msg(
                "The gift cannot be opened",
            )));
        }

        self.status = GiftStatus::Opened;
        Ok(())
    }

    pub fn set_janken_events(&mut self, win_event: JankenEventId, lose_event: JankenEventId) {
        self.janken_win_event = Some(win_event);
        self.janken_lose_event = Some(lose_event);
    }
}
