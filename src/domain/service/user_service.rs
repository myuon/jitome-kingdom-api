use crate::domain::interface::IUserRepository;
use crate::domain::model::{Authorization, User};
use crate::wrapper::error::ServiceError;
use serde::*;
use std::sync::Arc;

pub struct UserService {
    user_repo: Arc<dyn IUserRepository + Sync + Send>,
}

#[derive(Deserialize)]
pub struct UserCreateInput {
    screen_name: Option<String>,
    display_name: String,
}

impl UserService {
    pub fn new(user_repo: Arc<dyn IUserRepository + Sync + Send>) -> UserService {
        UserService { user_repo }
    }

    /*
    pub async fn create(
        &self,
        auth: Authorization,
        input: UserCreateInput,
    ) -> Result<(), ServiceError> {
        auth.require_auth()?;

        let user = User::new(input.screen_name, input.display_name);
        self.user_repo.create(user).await?;

        Ok(())
    }
    */

    pub async fn get_me(&self, auth: Authorization) -> Result<serde_json::Value, ServiceError> {
        let user = auth.require_auth()?;

        let message = format!("You're {}", user.user_id.0);
        let result = serde_json::json!({ "data": message });

        Ok(result)
    }
}
