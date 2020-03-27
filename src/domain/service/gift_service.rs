use crate::domain::interface::{IGiftRepository, IUserRepository};
use crate::domain::model::{Authorization, Gift, GiftId, GiftType};
use crate::wrapper::error::ServiceError;
use std::sync::Arc;

pub struct GiftService {
    gift_repository: Arc<dyn IGiftRepository + Sync + Send>,
    user_repository: Arc<dyn IUserRepository + Sync + Send>,
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

    pub async fn list(&self, auth: Authorization) -> Result<Vec<Gift>, ServiceError> {
        let auth_user = auth.require_auth()?;
        let user = self
            .user_repository
            .find_by_subject(&auth_user.subject)
            .await?;

        self.gift_repository.find_by_user_id(&user.id).await
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
