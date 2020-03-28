use crate::domain::interface::{IGiftRepository, IUserRepository};
use crate::domain::model::Authorization;
use crate::wrapper::error::ServiceError;
use std::sync::Arc;

pub struct GiftDistributionService {
    user_repo: Arc<dyn IUserRepository + Sync + Send>,
    gift_repo: Arc<dyn IGiftRepository + Sync + Send>,
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

    pub async fn distribute(&self, auth: Authorization) -> Result<(), ServiceError> {
        let auth_user = auth.require_auth()?;
        auth_user.require_admin()?;

        let users = self.user_repo.list().await?;
        warn!("Starting distribution to {:?} users...", users.len());

        Ok(())
    }
}
