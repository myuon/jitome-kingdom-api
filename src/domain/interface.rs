use crate::domain::model::{User, UserId};
use crate::wrapper::error::ServiceError;
use async_trait::async_trait;

#[async_trait]
pub trait IUserRepository {
    async fn find_by_id(&self, user_id: UserId) -> Result<User, ServiceError>;
    async fn create(&self, user: User) -> Result<(), ServiceError>;
    async fn save(&self, user: User) -> Result<(), ServiceError>;
}

pub trait IJWTHandler<Payload> {
    fn verify(&self, jwt: &str) -> Result<Payload, ServiceError>;
}
