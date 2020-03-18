use crate::domain::model::UserId;
use crate::wrapper::unixtime::UnixTime;

pub struct User {
    id: UserId,
    screen_name: Option<String>,
    display_name: String,
    point: u64, // みょんポイント
    created_at: UnixTime,
}

impl User {
    pub fn new(screen_name: Option<String>, display_name: String) -> Self {
        User {
            id: UserId::new(),
            screen_name,
            display_name,
            point: 0,
            created_at: UnixTime::now(),
        }
    }
}
