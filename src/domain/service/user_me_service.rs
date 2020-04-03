use crate::domain::interface::IUserRepository;
use crate::domain::model::{Authorization, Role, User};
use crate::wrapper::error::ServiceError;
use crate::wrapper::url::Url;
use serde::*;
use std::sync::Arc;

pub struct UserMeService {
    user_repo: Arc<dyn IUserRepository + Sync + Send>,
}

#[derive(Deserialize)]
pub struct UserCreateInput {
    screen_name: Option<String>,
    display_name: String,
}

#[derive(Deserialize)]
pub struct UpdateMeInput {
    picture_url: String,
    screen_name: String,
    display_name: String,
}

#[derive(Serialize)]
pub struct UserProfile {
    #[serde(flatten)]
    user: User,
    roles: Vec<Role>,
}

impl UserMeService {
    pub fn new(user_repo: Arc<dyn IUserRepository + Sync + Send>) -> UserMeService {
        UserMeService { user_repo }
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

    pub async fn get_me(&self, auth: Authorization) -> Result<UserProfile, ServiceError> {
        let auth_user = auth.require_auth()?;

        let user = self.ensure_user_created(&auth_user.subject).await?;
        Ok(UserProfile {
            user,
            roles: auth_user.roles,
        })
    }

    pub async fn update_me(
        &self,
        auth: Authorization,
        input: UpdateMeInput,
    ) -> Result<(), ServiceError> {
        let auth_user = auth.require_auth()?;

        let r = regex::Regex::new(r"^[a-zA-Z0-9_]{3,}$").unwrap();
        if !r.is_match(&input.screen_name) {
            return Err(ServiceError::bad_request(failure::err_msg(
                "screen_name does not match the policy",
            )));
        }

        let mut user = self.user_repo.find_by_subject(&auth_user.subject).await?;
        user.update(
            input.screen_name,
            input.display_name,
            Url(input.picture_url),
        );
        self.user_repo.save(user).await?;

        Ok(())
    }
}
