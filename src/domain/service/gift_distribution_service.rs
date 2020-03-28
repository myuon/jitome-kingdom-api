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

        for user_id in users {
            let gift = Gift::new(
                GiftType::Point(input.point),
                input.description.to_string(),
                user_id,
            );
            match self.gift_repo.create(gift).await {
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
