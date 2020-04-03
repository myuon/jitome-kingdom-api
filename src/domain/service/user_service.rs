use crate::domain::interface::IUserRepository;
use crate::domain::model::{Authorization, User};
use crate::wrapper::error::ServiceError;
use std::sync::Arc;

pub struct UserService {
    user_repository: Arc<dyn IUserRepository + Sync + Send>,
}

impl UserService {
    pub fn new(user_repository: Arc<dyn IUserRepository + Sync + Send>) -> Self {
        UserService { user_repository }
    }

    pub async fn find_by_screen_name(
        &self,
        auth: Authorization,
        screen_name: String,
    ) -> Result<User, ServiceError> {
        auth.require_auth()?;

        let user = self
            .user_repository
            .find_by_screen_name(&screen_name)
            .await?;
        Ok(user)
    }

    pub async fn is_screen_name_available(
        &self,
        auth: Authorization,
        screen_name: String,
    ) -> Result<serde_json::Value, ServiceError> {
        auth.require_auth()?;

        let available = match self.user_repository.find_by_screen_name(&screen_name).await {
            Ok(_) => false,
            Err(err) if err.status_code == http::StatusCode::NOT_FOUND => true,
            Err(err) => return Err(err),
        };

        Ok(serde_json::json!({
            "availability": available,
        }))
    }
}
