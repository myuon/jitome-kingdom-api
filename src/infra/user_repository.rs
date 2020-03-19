use crate::domain::interface::IUserRepository;
use crate::domain::model::{User, UserId};
use crate::infra::ConnPool;
use crate::wrapper::error::ServiceError;
use crate::wrapper::unixtime::UnixTime;
use async_trait::async_trait;
use debil::*;
use debil_mysql::*;
use std::sync::Arc;

#[derive(Table)]
#[sql(table_name = "user", sql_type = "MySQLValue", primary_key = "id")]
pub struct UserRecord {
    id: String,
    screen_name: Option<String>,
    display_name: String,
    point: u64,
    created_at: i64,
}

impl UserRecord {
    pub fn into_model(self) -> User {
        User {
            id: UserId(self.id),
            screen_name: self.screen_name,
            display_name: self.display_name,
            point: self.point,
            created_at: UnixTime(self.created_at),
        }
    }

    pub fn from_model(user: User) -> Self {
        UserRecord {
            id: user.id.0,
            screen_name: user.screen_name,
            display_name: user.display_name,
            point: user.point,
            created_at: user.created_at.0,
        }
    }
}

pub struct UserRepository {
    pool: Arc<ConnPool>,
}

impl UserRepository {
    pub fn new(pool: Arc<ConnPool>) -> Self {
        UserRepository { pool }
    }
}

#[async_trait]
impl IUserRepository for UserRepository {
    async fn find_by_id(&self, user_id: UserId) -> Result<(), ServiceError> {
        unimplemented!()
    }

    async fn create(&self, user: User) -> Result<(), ServiceError> {
        let mut conn = self.pool.get_conn().await?;
        conn.save(UserRecord::from_model(user)).await?;

        Ok(())
    }

    async fn save(&self, user: User) -> Result<(), ServiceError> {
        unimplemented!()
    }
}