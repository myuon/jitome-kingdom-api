use chrono::offset::TimeZone;
use chrono::{DateTime, NaiveDateTime};
use serde::*;

// UnixTime in JST
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UnixTime(pub i64);

impl UnixTime {
    pub fn now_jst() -> Self {
        UnixTime(
            chrono_tz::Asia::Tokyo
                .from_utc_datetime(&chrono::Utc::now().naive_utc())
                .timestamp(),
        )
    }

    pub fn datetime(&self) -> DateTime<chrono_tz::Tz> {
        chrono_tz::Asia::Tokyo.timestamp(self.0, 0)
    }
}
