use crate::domain::interface::IUserRepository;
use crate::domain::model::{User, UserId};
use crate::infra::ConnPool;
use crate::wrapper::error::ServiceError;
use crate::wrapper::unixtime::UnixTime;
use async_trait::async_trait;
use debil::*;
use debil_mysql::*;
use std::sync::Arc;

#[derive(Table, Clone)]
#[sql(table_name = "user", sql_type = "MySQLValue", primary_key = "id")]
pub struct UserRecord {
    #[sql(size = 100)]
    id: String,
    #[sql(size = 100)]
    screen_name: Option<String>,
    #[sql(size = 100)]
    display_name: String,
    point: u64,
    created_at: i64,
    #[sql(size = 100, unique = true)]
    subject: String,
}

impl UserRecord {
    pub fn into_model(self) -> User {
        User {
            id: UserId(self.id),
            screen_name: self.screen_name,
            display_name: self.display_name,
            point: self.point,
            created_at: UnixTime(self.created_at),
            subject: self.subject,
        }
    }

    pub fn from_model(user: User) -> Self {
        UserRecord {
            id: user.id.0,
            screen_name: user.screen_name,
            display_name: user.display_name,
            point: user.point,
            created_at: user.created_at.0,
            subject: user.subject,
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
    async fn find_by_id(&self, user_id: &UserId) -> Result<User, ServiceError> {
        let mut conn = self.pool.get_conn().await?;
        let user = conn
            .first_with::<UserRecord>(QueryBuilder::new().filter(format!(
                "{}.id = '{}'",
                table_name::<UserRecord>(),
                user_id.0
            )))
            .await?;

        Ok(user.into_model())
    }

    async fn find_by_subject(&self, subject: &str) -> Result<User, ServiceError> {
        let mut conn = self.pool.get_conn().await?;
        let user = conn
            .first_with::<UserRecord>(QueryBuilder::new().filter(format!(
                "{}.subject = '{}'",
                table_name::<UserRecord>(),
                subject
            )))
            .await?;

        Ok(user.into_model())
    }

    async fn create(&self, user: User) -> Result<(), ServiceError> {
        let mut conn = self.pool.get_conn().await?;
        conn.create(UserRecord::from_model(user)).await?;

        Ok(())
    }

    async fn save(&self, user: User) -> Result<(), ServiceError> {
        let mut conn = self.pool.get_conn().await?;
        conn.save(UserRecord::from_model(user)).await?;

        Ok(())
    }
}
