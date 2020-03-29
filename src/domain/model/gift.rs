use crate::domain::model::{GiftId, UserId};
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
    pub user_id: UserId,
    pub created_at: UnixTime,
    pub status: GiftStatus,
}

impl Gift {
    pub fn new(gift_type: GiftType, description: String, user_id: UserId) -> Self {
        Gift {
            id: GiftId::new(),
            gift_type,
            description,
            user_id,
            created_at: UnixTime::now(),
            status: GiftStatus::Ready,
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
}
