use crate::domain::interface::IUserRepository;
use crate::domain::model::{User, UserId};
use crate::infra::ConnPool;
use crate::wrapper::error::ServiceError;
use crate::wrapper::unixtime::UnixTime;
use crate::wrapper::url::Url;
use async_trait::async_trait;
use debil::*;
use debil_mysql::*;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Table, Clone, Accessor)]
#[sql(table_name = "user", sql_type = "MySQLValue", primary_key = "id")]
pub struct UserRecord {
    #[sql(size = 100)]
    id: String,
    #[sql(size = 100, unique = true)]
    screen_name: Option<String>,
    #[sql(size = 100)]
    display_name: String,
    point: u64,
    created_at: i64,
    #[sql(size = 100, unique = true)]
    subject: String,
    #[sql(size = 256)]
    picture_url: Option<String>,
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
            picture_url: self.picture_url.map(Url),
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
            picture_url: user.picture_url.map(|u| u.0),
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

struct UserIdMapper {
    id: String,
}

impl SQLMapper for UserIdMapper {
    type ValueType = debil_mysql::MySQLValue;

    fn map_from_sql(hm: HashMap<String, Self::ValueType>) -> Self {
        UserIdMapper {
            id: debil_mysql::MySQLValue::deserialize(hm[accessor!(UserRecord::id)].clone()),
        }
    }
}

#[async_trait]
impl IUserRepository for UserRepository {
    async fn list_id(&self) -> Result<Vec<UserId>, ServiceError> {
        let mut conn = self.pool.get_conn().await?;
        let users = conn
            .load_with2::<UserRecord, UserIdMapper>(
                QueryBuilder::new().selects(vec![accessor!(UserRecord::id)]),
            )
            .await?;

        Ok(users.into_iter().map(|m| UserId(m.id)).collect())
    }

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

    async fn find_by_screen_name(&self, screen_name: &String) -> Result<User, ServiceError> {
        let mut conn = self.pool.get_conn().await?;
        let user = conn
            .first_with::<UserRecord>(QueryBuilder::new().filter(format!(
                "{}.screen_name = '{}'",
                table_name::<UserRecord>(),
                screen_name
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

#[cfg(test)]
pub mod user_repository_mock {
    use super::*;
    use std::sync::Mutex;

    pub struct UserRepositoryStub {
        pub item: User,
        pub created: Arc<Mutex<Vec<User>>>,
        pub saved: Arc<Mutex<Vec<User>>>,
    }

    impl UserRepositoryStub {
        pub fn new(user: User) -> Self {
            UserRepositoryStub {
                item: user,
                created: Arc::new(Mutex::new(Vec::new())),
                saved: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    #[async_trait]
    impl IUserRepository for UserRepositoryStub {
        async fn list_id(&self) -> Result<Vec<UserId>, ServiceError> {
            unimplemented!()
        }

        async fn find_by_id(&self, user_id: &UserId) -> Result<User, ServiceError> {
            Ok(self.item.clone())
        }

        async fn find_by_subject(&self, subject: &str) -> Result<User, ServiceError> {
            Ok(self.item.clone())
        }

        async fn create(&self, user: User) -> Result<(), ServiceError> {
            self.created.lock().unwrap().push(user);

            Ok(())
        }

        async fn save(&self, user: User) -> Result<(), ServiceError> {
            self.saved.lock().unwrap().push(user);

            Ok(())
        }
    }

    pub struct UserRepositoryListIdStub {
        pub ids: Vec<UserId>,
    }

    impl UserRepositoryListIdStub {
        pub fn new(ids: Vec<UserId>) -> Self {
            UserRepositoryListIdStub { ids }
        }
    }

    #[async_trait]
    impl IUserRepository for UserRepositoryListIdStub {
        async fn list_id(&self) -> Result<Vec<UserId>, ServiceError> {
            Ok(self.ids.clone())
        }

        async fn find_by_id(&self, user_id: &UserId) -> Result<User, ServiceError> {
            unimplemented!()
        }

        async fn find_by_subject(&self, subject: &str) -> Result<User, ServiceError> {
            unimplemented!()
        }

        async fn create(&self, user: User) -> Result<(), ServiceError> {
            unimplemented!()
        }

        async fn save(&self, user: User) -> Result<(), ServiceError> {
            unimplemented!()
        }
    }
}
