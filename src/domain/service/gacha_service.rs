use crate::domain::interface::{IGachaEventRepository, IUserRepository};
use crate::domain::model::{Authorization, GachaEvent, GachaEventId, GachaType, User, UserId};
use crate::error::ServiceError;
use crate::wrapper::rand_gen::RandomGen;
use crate::wrapper::unixtime::UnixTime;
use std::sync::Arc;

// ガチャ
pub struct GachaService {
    gacha_repo: Arc<dyn IGachaEventRepository + Sync + Send>,
    user_repo: Arc<dyn IUserRepository + Sync + Send>,
}

impl GachaService {
    pub fn new(
        gacha_repo: Arc<dyn IGachaEventRepository + Sync + Send>,
        user_repo: Arc<dyn IUserRepository + Sync + Send>,
    ) -> GachaService {
        GachaService {
            gacha_repo,
            user_repo,
        }
    }

    pub async fn try_daily(&self, auth: Authorization) -> Result<serde_json::Value, ServiceError> {
        let auth_user = auth.require_auth()?;
        let mut user = self.user_repo.find_by_subject(&auth_user.subject).await?;
        let user_cloned = user.clone();

        let n = RandomGen::range(5, 15);
        user.add_point(n);
        self.user_repo.save(user.clone()).await?;

        let event = GachaEvent {
            id: GachaEventId::new(),
            user_id: user.id,
            gacha_type: GachaType::Daily,
            created_at: UnixTime::now(),
        };

        if let Err(err) = self.gacha_repo.create(event.clone()).await {
            warn!("Failed to create a new gacha event: {:?} {:?}", event, err);
            error!("{:?}", err);

            // 失敗したときはロールバックを試みる
            if let Err(err) = self.user_repo.save(user_cloned.clone()).await {
                // ロールバックに失敗した場合は不整合が起こるのでログだけ吐いておく
                error!("Failed to save the original user data: {:?}", user_cloned);
                error!("{:?}", err);

                return Err(ServiceError::internal_server_error(failure::err_msg(
                    "operation failed",
                )));
            }

            warn!("Rollback completed");
        };

        Ok(serde_json::json!({ "obtained": n }))
    }
}
