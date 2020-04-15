use crate::domain::interface::{IPointEventRepository, IUserRepository};
use crate::domain::model::PointEvent;
use crate::wrapper::error::ServiceError;
use std::sync::Arc;

#[derive(Clone)]
pub struct RankingProcessService {
    user_repo: Arc<dyn IUserRepository + Sync + Send>,
    point_repo: Arc<dyn IPointEventRepository + Sync + Send>,
}

impl RankingProcessService {
    pub fn new(
        user_repo: Arc<dyn IUserRepository + Sync + Send>,
        point_repo: Arc<dyn IPointEventRepository + Sync + Send>,
    ) -> Self {
        RankingProcessService {
            user_repo,
            point_repo,
        }
    }

    pub async fn run(&self) -> Result<(), ServiceError> {
        let user_ids = self.user_repo.list_id().await?;
        for user_id in user_ids {
            match self.point_repo.find_by_id(&user_id).await {
                Err(err) if err.status_code == http::StatusCode::NOT_FOUND => {
                    let user = self.user_repo.find_by_id(&user_id).await?;
                    self.point_repo
                        .save(PointEvent::new(user.id, user.point))
                        .await?;
                }
                Ok(mut event) => {
                    let user = self.user_repo.find_by_id(&user_id).await?;
                    event.update(user.point);
                    self.point_repo.save(event).await?;
                }
                Err(err) => return Err(err),
            }
        }

        Ok(())
    }
}
