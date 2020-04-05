use crate::domain::interface::{IGiftRepository, IUserRepository};
use crate::domain::model::{Authorization, Gift, GiftType};
use crate::wrapper::error::ServiceError;
use serde::*;
use std::sync::Arc;

pub struct GiftDistributionService {
    user_repo: Arc<dyn IUserRepository + Sync + Send>,
    gift_repo: Arc<dyn IGiftRepository + Sync + Send>,
}

#[derive(Deserialize)]
pub struct DistributeInput {
    point: u64,
    description: String,
}

impl GiftDistributionService {
    pub fn new(
        user_repo: Arc<dyn IUserRepository + Sync + Send>,
        gift_repo: Arc<dyn IGiftRepository + Sync + Send>,
    ) -> Self {
        GiftDistributionService {
            user_repo,
            gift_repo,
        }
    }

    // これはひどい
    pub async fn distribute_point(
        &self,
        auth: Authorization,
        input: DistributeInput,
    ) -> Result<(), ServiceError> {
        let auth_user = auth.require_auth()?;
        auth_user.require_admin()?;

        let users = self.user_repo.list_id().await?;
        info!("Starting distribution to {:?} users...", users.len());

        let gift = Gift::new(GiftType::Point(input.point), input.description.to_string());
        self.gift_repo.create(gift.clone()).await?;

        for user_id in users {
            match self
                .gift_repo
                .save_status(gift.id.clone(), user_id, gift.status.clone())
                .await
            {
                Err(err) => {
                    error!("Failed to create a gift, {:?}", err);
                    continue;
                }
                _ => (),
            }
        }

        info!("Distribution completed!");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::{AuthUser, Role, UserId};
    use crate::infra::gift_repository_mock::GiftRepositoryMock;
    use crate::infra::user_repository_mock::UserRepositoryListIdStub;

    #[tokio::test]
    async fn distribute_point_requires_admin() -> Result<(), ServiceError> {
        let user_repo = Arc::new(UserRepositoryListIdStub::new(vec![
            UserId::new(),
            UserId::new(),
            UserId::new(),
            UserId::new(),
        ]));
        let gift_repo = Arc::new(GiftRepositoryMock::new());
        let service = GiftDistributionService::new(user_repo.clone(), gift_repo.clone());

        let err = service
            .distribute_point(
                Authorization::new(Ok(AuthUser {
                    subject: "".to_string(),
                    roles: vec![],
                })),
                DistributeInput {
                    point: 0,
                    description: "".to_string(),
                },
            )
            .await
            .expect_err("error");

        assert_eq!(err.status_code, http::StatusCode::UNAUTHORIZED);

        Ok(())
    }

    #[tokio::test]
    async fn distribute_point() -> Result<(), ServiceError> {
        let user_repo = Arc::new(UserRepositoryListIdStub::new(vec![
            UserId::new(),
            UserId::new(),
            UserId::new(),
            UserId::new(),
        ]));
        let gift_repo = Arc::new(GiftRepositoryMock::new());
        let service = GiftDistributionService::new(user_repo.clone(), gift_repo.clone());

        service
            .distribute_point(
                Authorization::new(Ok(AuthUser {
                    subject: "1".to_string(),
                    roles: vec![Role::Admin],
                })),
                DistributeInput {
                    point: 100,
                    description: "hoge piyo".to_string(),
                },
            )
            .await?;

        let created = gift_repo.created.lock().unwrap().clone();
        assert_eq!(created.len(), 4);
        assert_eq!(created[0].gift_type, GiftType::Point(100));

        Ok(())
    }
}
