use crate::domain::model::User;
use serde::*;

#[derive(Serialize)]
pub struct PointRankingRecord {
    #[serde(flatten)]
    pub user: User,
}

impl PointRankingRecord {
    pub fn new(user: User) -> Self {
        PointRankingRecord { user }
    }
}

#[derive(Serialize)]
pub struct PointDiffRankingRecord {
    #[serde(flatten)]
    pub user: User,
    pub diff: u64,
}

impl PointDiffRankingRecord {
    pub fn new(user: User, diff: u64) -> Self {
        PointDiffRankingRecord { user, diff }
    }
}
