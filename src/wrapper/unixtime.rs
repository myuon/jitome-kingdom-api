use chrono::offset::TimeZone;
use chrono::{DateTime, NaiveDateTime};
use serde::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UnixTime(pub i64);

impl UnixTime {
    pub fn now() -> Self {
        UnixTime(chrono::Utc::now().timestamp())
    }

    pub fn datetime_jp(&self) -> DateTime<chrono_tz::Tz> {
        chrono_tz::Asia::Tokyo.from_utc_datetime(&NaiveDateTime::from_timestamp(self.0, 0))
    }
}
