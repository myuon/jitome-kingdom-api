use crate::domain::interface::{IPointEventRepository, IUserRepository};
use crate::domain::model::PointEvent;
use crate::wrapper::error::ServiceError;
use crate::wrapper::unixtime::UnixTime;
use serde::*;
use std::sync::Arc;

#[derive(Clone)]
pub struct PointProcessService {
    user_repo: Arc<dyn IUserRepository + Sync + Send>,
    point_repo: Arc<dyn IPointEventRepository + Sync + Send>,
}

#[derive(Serialize)]
pub struct StartProcessOutput {
    executed: bool,
}

impl PointProcessService {
    pub fn new(
        user_repo: Arc<dyn IUserRepository + Sync + Send>,
        point_repo: Arc<dyn IPointEventRepository + Sync + Send>,
    ) -> Self {
        PointProcessService {
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

    pub async fn start(&self) -> Result<StartProcessOutput, ServiceError> {
        let may_user_id = self.user_repo.find_any_user().await?;
        let user_id = match may_user_id {
            None => return Ok(StartProcessOutput { executed: false }),
            Some(user_id) => user_id,
        };
        let point = self.point_repo.find_by_id(&user_id).await?;

        // 23時間より短い間隔でリトライはしない
        if (UnixTime::now().datetime_jst() - point.updated_at.datetime_jst()).num_hours() < 23 {
            return Ok(StartProcessOutput { executed: false });
        }

        self.run().await?;

        Ok(StartProcessOutput { executed: true })
    }
}
