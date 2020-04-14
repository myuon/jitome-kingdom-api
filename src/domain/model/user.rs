use crate::domain::model::UserId;
use crate::unixtime::UnixTime;
use crate::url::Url;
use serde::*;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct User {
    pub id: UserId,
    pub screen_name: Option<String>,
    pub display_name: String,
    // みょんポイント
    pub point: u64,
    pub created_at: UnixTime,
    pub subject: String,
    pub picture_url: Option<Url>,
    // 最後にデイリーガチャを引いた時刻
    pub last_tried_daily_gacha: UnixTime,
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
            last_tried_daily_gacha: UnixTime(0),
        }
    }

    pub fn add_point(&mut self, p: u64) {
        self.point += p;
    }

    pub fn subtract_point(&mut self, p: u64) {
        self.point -= p;
    }

    pub fn update(&mut self, screen_name: String, display_name: String, picture_url: Url) {
        self.screen_name = Some(screen_name);
        self.display_name = display_name;
        self.picture_url = Some(picture_url);
    }

    pub fn update_daily_gacha_timestamp(&mut self) -> UnixTime {
        let prev = self.last_tried_daily_gacha.clone();
        self.last_tried_daily_gacha = UnixTime::now();

        prev
    }

    pub fn is_daily_gacha_available_at(&self, target_time: UnixTime) -> bool {
        // デイリーガチャなので、日付が違ったら引いても良い
        self.last_tried_daily_gacha.datetime_jst().date() != target_time.datetime_jst().date()
    }
}

#[test]
fn gacha_event_is_available_at() {
    let ev = User {
        id: UserId::new(),
        last_tried_daily_gacha: UnixTime(0),
        ..Default::default()
    };

    assert!(!ev.is_daily_gacha_available_at(UnixTime(1)));
    assert!(ev.is_daily_gacha_available_at(UnixTime(89400)));
}
