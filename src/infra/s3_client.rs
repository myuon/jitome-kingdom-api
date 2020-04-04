use crate::wrapper::error::ServiceError;
use rusoto_s3::S3;

#[derive(Clone)]
pub struct S3Client {
    client: rusoto_s3::S3Client,
}

impl S3Client {
    pub fn new(region: rusoto_core::Region) -> Self {
        S3Client {
            client: rusoto_s3::S3Client::new(region),
        }
    }

    pub async fn put_object_public(
        &self,
        bucket_name: String,
        key: String,
        body: Vec<u8>,
    ) -> Result<(), ServiceError> {
        self.client
            .put_object(rusoto_s3::PutObjectRequest {
                acl: Some("public-read".to_string()),
                bucket: bucket_name,
                key,
                body: Some(From::from(body)),
                ..Default::default()
            })
            .await?;

        Ok(())
    }
}
