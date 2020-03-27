use crate::domain::model::{GiftId, UserId};
use crate::wrapper::error::ServiceError;
use crate::wrapper::unixtime::UnixTime;

#[derive(Clone, Debug)]
pub enum GiftType {
    Point(u64),
}

#[derive(Clone, Debug, PartialOrd, PartialEq)]
pub enum GiftStatus {
    Ready,
    Opened,
}

#[derive(Clone, Debug)]
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
