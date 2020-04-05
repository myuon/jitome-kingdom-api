use crate::domain::model::{JankenEventId, UserId};
use crate::unixtime::UnixTime;
use crate::wrapper::error::ServiceError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Clone, Debug)]
pub enum JankenHand {
    Rock,
    Paper,
    Scissors,
}

impl JankenHand {
    pub fn to_string(&self) -> String {
        use JankenHand::*;

        match self {
            Rock => "rock",
            Paper => "paper",
            Scissors => "scissors",
        }
        .to_string()
    }

    pub fn from_str(rep: &str) -> Result<Self, ServiceError> {
        match rep {
            "rock" => Ok(JankenHand::Rock),
            "paper" => Ok(JankenHand::Paper),
            "scissors" => Ok(JankenHand::Scissors),
            _ => Err(ServiceError::bad_request(failure::err_msg(format!(
                "Unsupported hand: {}",
                rep
            )))),
        }
    }
}

impl Serialize for JankenHand {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for JankenHand {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer).and_then(|s| {
            JankenHand::from_str(&s).map_err(|err| serde::de::Error::custom(err.error))
        })
    }
}

#[derive(Clone, Debug)]
pub enum JankenStatus {
    Ready,
    Won,
    Lost,
}

impl JankenStatus {
    pub fn to_string(&self) -> String {
        use JankenStatus::*;

        match self {
            Ready => "ready",
            Won => "won",
            Lost => "lost",
        }
        .to_string()
    }

    pub fn from_str(rep: &str) -> Result<Self, ServiceError> {
        match rep {
            "ready" => Ok(JankenStatus::Ready),
            "won" => Ok(JankenStatus::Won),
            "lost" => Ok(JankenStatus::Lost),
            _ => Err(ServiceError::bad_request(failure::err_msg(format!(
                "Unsupported status: {}",
                rep
            )))),
        }
    }
}

impl Serialize for JankenStatus {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for JankenStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer).and_then(|s| {
            JankenStatus::from_str(&s).map_err(|err| serde::de::Error::custom(err.error))
        })
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct JankenEvent {
    pub id: JankenEventId,
    pub user_id: UserId,
    pub hand: JankenHand,
    pub created_at: UnixTime,
    pub status: JankenStatus,
}

impl JankenEvent {
    pub fn new(user_id: UserId, hand: JankenHand) -> JankenEvent {
        JankenEvent {
            id: JankenEventId::new(),
            user_id,
            hand,
            created_at: UnixTime::now(),
            status: JankenStatus::Ready,
        }
    }
}
