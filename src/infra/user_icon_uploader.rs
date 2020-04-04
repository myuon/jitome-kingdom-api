use crate::domain::interface::IUserIconUploader;
use crate::domain::model::UserId;
use crate::infra::S3Client;
use crate::url::Url;
use crate::wrapper::base64::Base64;
use crate::wrapper::error::ServiceError;
use async_trait::async_trait;
use std::sync::Arc;

#[derive(Clone)]
pub struct ImageId(pub String);

impl ImageId {
    pub fn new() -> Self {
        ImageId(uuid::Uuid::new_v4().to_string())
    }
}

pub struct UserIconUploader {
    s3_client: Arc<S3Client>,
    bucket_name: String,
}

impl UserIconUploader {
    pub fn new(s3_client: Arc<S3Client>, bucket_name: String) -> Self {
        UserIconUploader {
            s3_client,
            bucket_name,
        }
    }

    fn path_for_icon(user_id: &UserId, key_id: &ImageId) -> String {
        format!("public/{}/{}", user_id.0, key_id.0)
    }
}

#[async_trait]
impl IUserIconUploader for UserIconUploader {
    async fn upload_user_icon(&self, user_id: &UserId, image: Base64) -> Result<Url, ServiceError> {
        let image_id = ImageId::new();
        let image_data = image.decode()?;

        self.s3_client
            .put_object_public(
                self.bucket_name.clone(),
                UserIconUploader::path_for_icon(&user_id, &image_id),
                image_data,
            )
            .await?;

        Ok(Url(format!(
            "https://{}.s3.amazonaws.com/{}",
            self.bucket_name,
            UserIconUploader::path_for_icon(&user_id, &image_id)
        )))
    }
}
