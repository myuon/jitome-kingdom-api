use crate::domain::interface::{IGiftRepository, IUserRepository};
use crate::domain::model::{Authorization, Gift, GiftId, GiftStatus, GiftType};
use crate::wrapper::error::ServiceError;
use serde::*;
use std::sync::Arc;

pub struct GiftService {
    gift_repository: Arc<dyn IGiftRepository + Sync + Send>,
    user_repository: Arc<dyn IUserRepository + Sync + Send>,
}

#[derive(Serialize)]
pub struct ListGiftResponse {
    data: Vec<Gift>,
}

impl GiftService {
    pub fn new(
        gift_repository: Arc<dyn IGiftRepository + Sync + Send>,
        user_repository: Arc<dyn IUserRepository + Sync + Send>,
    ) -> Self {
        GiftService {
            gift_repository,
            user_repository,
        }
    }

    pub async fn list_by_status(
        &self,
        auth: Authorization,
        status: GiftStatus,
    ) -> Result<ListGiftResponse, ServiceError> {
        let auth_user = auth.require_auth()?;
        let user = self
            .user_repository
            .find_by_subject(&auth_user.subject)
            .await?;

        let gifts = self
            .gift_repository
            .find_by_user_id_status(&user.id, status)
            .await?;

        Ok(ListGiftResponse { data: gifts })
    }

    pub async fn open(&self, auth: Authorization, gift_id: &GiftId) -> Result<(), ServiceError> {
        let auth_user = auth.require_auth()?;
        let mut user = self
            .user_repository
            .find_by_subject(&auth_user.subject)
            .await?;

        let mut gift = self.gift_repository.find_by_id(gift_id).await?;
        if user.id != gift.user_id {
            return Err(ServiceError::unauthorized(failure::err_msg(
                "access_denied",
            )));
        }

        gift.open()?;
        match gift.gift_type {
            GiftType::Point(p) => {
                user.add_point(p);
                self.user_repository.save(user).await?;
            }
        }

        self.gift_repository.save(gift).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::{GiftStatus, User, UserId};
    use crate::infra::gift_repository_mock::GiftRepositoryItemStub;
    use crate::infra::user_repository_mock::UserRepositoryStub;

    #[tokio::test]
    async fn cannot_open_not_for_me() -> Result<(), ServiceError> {
        let user = User {
            id: UserId("AAA".to_string()),
            ..Default::default()
        };
        let gift = Gift::new(
            GiftType::Point(5),
            "".to_string(),
            UserId("BBB".to_string()),
        );

        let gift_repo = Arc::new(GiftRepositoryItemStub::new(gift.clone()));
        let user_repo = Arc::new(UserRepositoryStub::new(user));
        let service = GiftService::new(gift_repo, user_repo);

        let err = service
            .open(Authorization::new(Ok(Default::default())), &gift.id)
            .await
            .expect_err("error");

        assert_eq!(err.status_code, http::StatusCode::UNAUTHORIZED);

        Ok(())
    }

    #[tokio::test]
    async fn open_gift_and_got_point() -> Result<(), ServiceError> {
        let user = User {
            point: 10,
            ..Default::default()
        };
        let gift = Gift::new(GiftType::Point(5), "".to_string(), user.id.clone());

        let gift_repo = Arc::new(GiftRepositoryItemStub::new(gift.clone()));
        let user_repo = Arc::new(UserRepositoryStub::new(user));
        let service = GiftService::new(gift_repo.clone(), user_repo.clone());

        service
            .open(Authorization::new(Ok(Default::default())), &gift.id)
            .await?;

        let gifts = gift_repo.saved.lock().unwrap().clone();
        assert_eq!(gifts.len(), 1);
        assert_eq!(gifts[0].status, GiftStatus::Opened);

        let users = user_repo.saved.lock().unwrap().clone();
        assert_eq!(users.len(), 1);
        assert_eq!(users[0].point, 15);

        Ok(())
    }
}
