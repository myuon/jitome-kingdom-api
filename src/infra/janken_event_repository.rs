use crate::domain::interface::IJankenEventRepository;
use crate::domain::model::{JankenEvent, JankenEventId, JankenHand, JankenStatus, UserId};
use crate::infra::ConnPool;
use crate::wrapper::error::ServiceError;
use crate::wrapper::unixtime::UnixTime;
use async_trait::async_trait;
use debil::*;
use debil_mysql::*;
use std::sync::Arc;

#[derive(Table, Clone, Accessor)]
#[sql(
    table_name = "janken_event",
    sql_type = "MySQLValue",
    primary_key = "id"
)]
pub struct JankenEventRecord {
    #[sql(size = 100)]
    id: String,
    #[sql(size = 100)]
    user_id: String,
    #[sql(size = 50)]
    hand: String,
    created_at: i64,
    #[sql(size = 50)]
    status: String,
}

impl JankenEventRecord {
    pub fn from_model(model: JankenEvent) -> Result<Self, ServiceError> {
        Ok(JankenEventRecord {
            id: model.id.0,
            user_id: model.user_id.0,
            hand: model.hand.to_string(),
            created_at: 0,
            status: model.status.to_string(),
        })
    }

    pub fn into_model(self) -> Result<JankenEvent, ServiceError> {
        Ok(JankenEvent {
            id: JankenEventId(self.id),
            user_id: UserId(self.user_id),
            hand: JankenHand::from_str(&self.hand)?,
            created_at: UnixTime(self.created_at),
            status: JankenStatus::from_str(&self.status)?,
        })
    }
}

pub struct JankenEventRepository {
    pool: Arc<ConnPool>,
}

impl JankenEventRepository {
    pub fn new(pool: Arc<ConnPool>) -> Self {
        JankenEventRepository { pool }
    }
}

#[async_trait]
impl IJankenEventRepository for JankenEventRepository {
    async fn find_by_user_id_status(
        &self,
        user_id: &UserId,
        status: JankenStatus,
    ) -> Result<Vec<JankenEvent>, ServiceError> {
        let mut conn = self.pool.get_conn().await?;
        let records = conn
            .load_with::<JankenEventRecord>(debil::QueryBuilder::new().filter(format!(
                "{}.{} = '{}' and {}.{} = '{}'",
                table_name::<JankenEventRecord>(),
                accessor!(JankenEventRecord::user_id),
                user_id.0,
                table_name::<JankenEventRecord>(),
                accessor!(JankenEventRecord::status),
                status.to_string()
            )))
            .await?;

        records.into_iter().map(|r| r.into_model()).collect()
    }

    async fn find_by_user_id(&self, user_id: &UserId) -> Result<Vec<JankenEvent>, ServiceError> {
        let mut conn = self.pool.get_conn().await?;
        let records = conn
            .load_with::<JankenEventRecord>(debil::QueryBuilder::new().filter(format!(
                "{}.{} = '{}'",
                table_name::<JankenEventRecord>(),
                accessor!(JankenEventRecord::user_id),
                user_id.0,
            )))
            .await?;

        records.into_iter().map(|r| r.into_model()).collect()
    }

    async fn create(&self, janken_event: JankenEvent) -> Result<(), ServiceError> {
        let mut conn = self.pool.get_conn().await?;
        let record = JankenEventRecord::from_model(janken_event)?;

        conn.create(record).await?;

        Ok(())
    }
}

#[cfg(test)]
pub mod janken_event_repository_mock {
    use super::*;
    use crate::domain::interface::IJankenEventRepository;
    use crate::domain::model::{JankenStatus, UserId};
    use std::sync::Mutex;

    pub struct JankenEventRepositoryMock {
        pub events: Vec<JankenEvent>,
        pub created: Arc<Mutex<Vec<JankenEvent>>>,
    }

    impl JankenEventRepositoryMock {
        pub fn new(events: Vec<JankenEvent>) -> Self {
            JankenEventRepositoryMock {
                events,
                created: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    #[async_trait]
    impl IJankenEventRepository for JankenEventRepositoryMock {
        async fn find_by_user_id_status(
            &self,
            user_id: &UserId,
            status: JankenStatus,
        ) -> Result<Vec<JankenEvent>, ServiceError> {
            Ok(self.events.clone())
        }

        async fn find_by_user_id(
            &self,
            user_id: &UserId,
        ) -> Result<Vec<JankenEvent>, ServiceError> {
            Ok(self.events.clone())
        }

        async fn create(&self, janken_event: JankenEvent) -> Result<(), ServiceError> {
            self.created.lock().unwrap().push(janken_event);

            Ok(())
        }
    }
}
