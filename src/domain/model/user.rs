use crate::domain::model::UserId;
use crate::unixtime::UnixTime;
use crate::url::Url;
use serde::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct User {
    pub id: UserId,
    pub screen_name: Option<String>,
    pub display_name: String,
    pub point: u64, // みょんポイント
    pub created_at: UnixTime,
    pub subject: String,
    pub picture_url: Option<Url>,
}

impl User {
    pub fn new(
        subject: String,
        screen_name: Option<String>,
        display_name: String,
        picture_url: Option<Url>,
    ) -> Self {
        User {
            id: UserId::new(),
            screen_name,
            display_name,
            point: 0,
            created_at: UnixTime::now(),
            subject,
            picture_url,
        }
    }

    pub fn add_point(&mut self, p: u64) {
        self.point += p;
    }
}
