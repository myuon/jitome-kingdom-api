use crate::domain::interface::IRankingRepository;
use crate::domain::model::{Authorization, PointDiffRankingRecord};
use crate::error::ServiceError;
use std::sync::Arc;

pub struct PointRankingService {
    ranking_repo: Arc<dyn IRankingRepository + Sync + Send>,
}

impl PointRankingService {
    pub fn new(ranking_repo: Arc<dyn IRankingRepository + Sync + Send>) -> Self {
        PointRankingService { ranking_repo }
    }

    pub async fn list_by_points(
        &self,
        auth: Authorization,
    ) -> Result<Vec<PointDiffRankingRecord>, ServiceError> {
        auth.require_auth()?;

        self.ranking_repo.list_top_points(10).await
    }

    pub async fn list_by_diff(
        &self,
        auth: Authorization,
    ) -> Result<Vec<PointDiffRankingRecord>, ServiceError> {
        auth.require_auth()?;

        self.ranking_repo.list_top_point_diffs(10).await
    }
}
