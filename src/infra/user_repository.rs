use crate::domain::interface::IUserRepository;
use crate::domain::model::{User, UserId};
use crate::wrapper::error::ServiceError;
use async_trait::async_trait;
use debil::*;
use debil_mysql::*;

#[derive(Table)]
#[sql(table_name = "user", sql_type = "MySQLValue", primary_key = "id")]
pub struct UserRecord {
    id: String,
    screen_name: Option<String>,
    display_name: String,
    point: u64,
    created_at: i64,
}

pub struct UserRepository {}

impl UserRepository {
    pub fn new() -> Self {
        UserRepository {}
    }
}

#[async_trait]
impl IUserRepository for UserRepository {
    async fn find_by_id(&self, user_id: UserId) -> Result<(), ServiceError> {
        unimplemented!()
    }

    async fn create(&self, user: User) -> Result<(), ServiceError> {
        unimplemented!()
    }

    async fn save(&self, user: User) -> Result<(), ServiceError> {
        unimplemented!()
    }
}
