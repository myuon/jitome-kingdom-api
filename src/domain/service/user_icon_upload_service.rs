use crate::domain::interface::{IUserIconUploader, IUserRepository};
use crate::domain::model::Authorization;
use crate::wrapper::base64::Base64;
use crate::wrapper::error::ServiceError;
use serde::*;
use std::sync::Arc;

pub struct UserIconUploadService {
    user_repo: Arc<dyn IUserRepository + Sync + Send>,
    user_icon_uploader: Arc<dyn IUserIconUploader + Sync + Send>,
}

#[derive(Deserialize)]
pub struct UploadInput {
    data: Base64,
}

impl UserIconUploadService {
    pub fn new(
        user_repo: Arc<dyn IUserRepository + Sync + Send>,
        user_icon_uploader: Arc<dyn IUserIconUploader + Sync + Send>,
    ) -> Self {
        UserIconUploadService {
            user_repo,
            user_icon_uploader,
        }
    }

    pub async fn upload(
        &self,
        auth: Authorization,
        input: UploadInput,
    ) -> Result<serde_json::Value, ServiceError> {
        let auth_user = auth.require_auth()?;

        let mut user = self.user_repo.find_by_subject(&auth_user.subject).await?;
        let url = self
            .user_icon_uploader
            .upload_user_icon(&user.id, input.data)
            .await?;
        user.picture_url = Some(url.clone());
        self.user_repo.save(user).await?;

        Ok(serde_json::json!({ "url": url }))
    }
}
