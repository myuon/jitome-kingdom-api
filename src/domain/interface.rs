use crate::base64::Base64;
use crate::domain::model::{GachaEvent, GachaType, Gift, GiftId, GiftStatus, User, UserId};
use crate::url::Url;
use crate::wrapper::error::ServiceError;
use async_trait::async_trait;

#[async_trait]
pub trait IUserRepository {
    async fn list_id(&self) -> Result<Vec<UserId>, ServiceError>;
    async fn find_by_id(&self, user_id: &UserId) -> Result<User, ServiceError>;
    async fn find_by_screen_name(&self, screen_name: &String) -> Result<User, ServiceError>;
    async fn find_by_subject(&self, subject: &str) -> Result<User, ServiceError>;
    async fn create(&self, user: User) -> Result<(), ServiceError>;
    async fn save(&self, user: User) -> Result<(), ServiceError>;
}

#[async_trait]
pub trait IGachaEventRepository {
    async fn find_by_user_type(
        &self,
        user_id: &UserId,
        gacha_type: &GachaType,
    ) -> Result<GachaEvent, ServiceError>;
    async fn create(&self, event: GachaEvent) -> Result<(), ServiceError>;
}

#[async_trait]
pub trait IGiftRepository {
    async fn find_by_id(&self, gift_id: &GiftId, user_id: &UserId) -> Result<Gift, ServiceError>;
    async fn find_by_user_id_status(
        &self,
        user_id: &UserId,
        status: GiftStatus,
    ) -> Result<Vec<Gift>, ServiceError>;
    async fn create(&self, gift: Gift) -> Result<(), ServiceError>;
    async fn save_status(
        &self,
        gift_id: GiftId,
        user_id: UserId,
        status: GiftStatus,
    ) -> Result<(), ServiceError>;
}

#[async_trait]
pub trait IUserIconUploader {
    async fn upload_user_icon(&self, user_id: &UserId, image: Base64) -> Result<Url, ServiceError>;
}
