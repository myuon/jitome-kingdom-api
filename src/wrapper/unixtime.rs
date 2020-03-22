use chrono::{DateTime, NaiveDateTime};
use serde::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UnixTime(pub i64);

impl UnixTime {
    pub fn now() -> Self {
        UnixTime(chrono::Local::now().timestamp())
    }

    pub fn datetime_jp(&self) -> DateTime<chrono_tz::Tz> {
        use chrono::offset::TimeZone;

        chrono_tz::Asia::Tokyo
            .from_local_datetime(&NaiveDateTime::from_timestamp(self.0, 0))
            .unwrap()
    }
}
