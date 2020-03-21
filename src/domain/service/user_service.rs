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

    async fn ensure_user_created(&self, subject: &str) -> Result<User, ServiceError> {
        let user = match self.user_repo.find_by_subject(subject).await {
            Err(err) if err.status_code == http::StatusCode::NOT_FOUND => {
                // 存在しなければ作成する
                let user = User::new(subject.to_string(), None, "no name".to_string(), None);
                self.user_repo.create(user.clone()).await?;

                Ok(user)
            }
            r => r,
        }?;

        Ok(user)
    }

    pub async fn get_me(&self, auth: Authorization) -> Result<serde_json::Value, ServiceError> {
        let auth_user = auth.require_auth()?;

        let user = self.ensure_user_created(&auth_user.subject).await?;
        let result = serde_json::json!({ "user": user });

        Ok(result)
    }
}
