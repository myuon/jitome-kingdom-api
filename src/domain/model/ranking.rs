use crate::domain::model::{User, UserId};
use crate::url::Url;
use serde::*;

#[derive(Serialize)]
pub struct UserInfo {
    user_id: UserId,
    screen_name: Option<String>,
    display_name: String,
    point: u64,
    picture_url: Option<Url>,
}

impl UserInfo {
    pub fn new(user: User) -> Self {
        UserInfo {
            user_id: user.id,
            screen_name: user.screen_name,
            display_name: user.display_name,
            point: user.point,
            picture_url: user.picture_url,
        }
    }
}

#[derive(Serialize)]
pub struct PointDiffRankingRecord {
    #[serde(flatten)]
    pub user: UserInfo,
    pub current: u64,
    pub diff: i64,
}

impl PointDiffRankingRecord {
    pub fn new(user: User, current: u64, diff: i64) -> Self {
        PointDiffRankingRecord {
            user: UserInfo::new(user),
            current,
            diff,
        }
    }
}
