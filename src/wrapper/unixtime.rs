use chrono::offset::TimeZone;
use chrono::DateTime;
use serde::*;

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialOrd, PartialEq)]
pub struct UnixTime(pub i64);

impl UnixTime {
    pub fn now() -> Self {
        // unixtimeは常に1970/1/1 0:0:0 in UTCからの秒数なのでtimezoneは関係ない
        UnixTime(chrono::Utc::now().timestamp())
    }

    pub fn datetime_jst(&self) -> DateTime<chrono_tz::Tz> {
        // DateTimeに変換するときは明示的にtimezoneを指定する必要がある
        chrono_tz::Asia::Tokyo.timestamp(self.0, 0)
    }
}
