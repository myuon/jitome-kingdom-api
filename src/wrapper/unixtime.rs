use serde::*;

#[derive(Serialize, Deserialize, Clone)]
pub struct UnixTime(pub i64);

impl UnixTime {
    pub fn now() -> Self {
        UnixTime(chrono::Local::now().timestamp())
    }
}
