use crate::domain::model::UserId;
use crate::wrapper::unixtime::UnixTime;

#[derive(Clone, Debug)]
pub struct PointEvent {
    pub user_id: UserId,
    pub current: u64,
    pub previous: Option<u64>,
    pub updated_at: UnixTime,
}

impl PointEvent {
    pub fn new(user_id: UserId, current: u64) -> Self {
        PointEvent {
            user_id,
            current,
            previous: None,
            updated_at: UnixTime::now(),
        }
    }

    pub fn update(&mut self, current: u64) {
        self.previous = Some(self.current);
        self.current = current;
        self.updated_at = UnixTime::now();
    }
}
