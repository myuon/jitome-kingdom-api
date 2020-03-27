use crate::domain::model::{GachaEvent, GachaType, Gift, GiftId, User, UserId};
use crate::wrapper::error::ServiceError;
use async_trait::async_trait;

#[async_trait]
pub trait IUserRepository {
    async fn find_by_id(&self, user_id: &UserId) -> Result<User, ServiceError>;
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
    async fn find_by_id(&self, gift_id: &GiftId) -> Result<Gift, ServiceError>;
    async fn find_by_user_id(&self, user_id: &UserId) -> Result<Vec<Gift>, ServiceError>;
    async fn create(&self, gift: Gift) -> Result<(), ServiceError>;
    async fn save(&self, gift: Gift) -> Result<(), ServiceError>;
}
