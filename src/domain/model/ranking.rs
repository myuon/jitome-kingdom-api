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
pub struct PointRankingRecord {
    #[serde(flatten)]
    pub user: UserInfo,
}

impl PointRankingRecord {
    pub fn new(user: User) -> Self {
        PointRankingRecord {
            user: UserInfo::new(user),
        }
    }
}

#[derive(Serialize)]
pub struct PointDiffRankingRecord {
    #[serde(flatten)]
    pub user: UserInfo,
    pub diff: u64,
}

impl PointDiffRankingRecord {
    pub fn new(user: User, diff: u64) -> Self {
        PointDiffRankingRecord {
            user: UserInfo::new(user),
            diff,
        }
    }
}
