use crate::domain::interface::IGiftRepository;
use crate::domain::model::{Gift, GiftId, GiftStatus, UserId};
use crate::infra::ConnPool;
use crate::wrapper::error::ServiceError;
use crate::wrapper::unixtime::UnixTime;
use async_trait::async_trait;
use debil::*;
use debil_mysql::*;
use std::sync::Arc;

#[derive(Table, Clone, Accessor)]
#[sql(table_name = "gift", sql_type = "MySQLValue", primary_key = "id")]
pub struct GiftRecord {
    #[sql(size = 100)]
    pub id: String,
    pub gift_type: String,
    pub description: String,
    #[sql(size = 100)]
    pub user_id: String,
    pub created_at: i64,
    #[sql(size = 50)]
    pub status: String,
}

impl GiftRecord {
    pub fn from_model(model: Gift) -> Result<Self, ServiceError> {
        Ok(GiftRecord {
            id: model.id.0,
            gift_type: serde_json::to_string(&model.gift_type)?,
            description: model.description,
            user_id: model.user_id.0,
            created_at: model.created_at.0,
            status: model.status.to_string(),
        })
    }

    pub fn into_model(self) -> Result<Gift, ServiceError> {
        Ok(Gift {
            id: GiftId(self.id),
            gift_type: serde_json::from_str(&self.gift_type)?,
            description: self.description,
            user_id: UserId(self.user_id),
            created_at: UnixTime(self.created_at),
            status: GiftStatus::from_str(&self.status),
        })
    }
}

pub struct GiftRepository {
    pool: Arc<ConnPool>,
}

impl GiftRepository {
    pub fn new(pool: Arc<ConnPool>) -> Self {
        GiftRepository { pool }
    }
}

#[async_trait]
impl IGiftRepository for GiftRepository {
    async fn find_by_id(&self, gift_id: &GiftId) -> Result<Gift, ServiceError> {
        let mut conn = self.pool.get_conn().await?;
        let records = conn
            .first_with::<GiftRecord>(debil::QueryBuilder::new().filter(format!(
                "{}.{} = '{:?}'",
                table_name::<GiftRecord>(),
                accessor!(GiftRecord::id),
                gift_id
            )))
            .await?;

        records.into_model()
    }

    async fn find_by_user_id_status(
        &self,
        user_id: &UserId,
        status: GiftStatus,
    ) -> Result<Vec<Gift>, ServiceError> {
        let mut conn = self.pool.get_conn().await?;
        let records = conn
            .load_with::<GiftRecord>(debil::QueryBuilder::new().filter(format!(
                "{}.{} = '{}' AND {}.{} = '{}'",
                table_name::<GiftRecord>(),
                accessor!(GiftRecord::user_id),
                user_id.0,
                table_name::<GiftRecord>(),
                accessor!(GiftRecord::status),
                status.to_string()
            )))
            .await?;

        records
            .into_iter()
            .map(|record| record.into_model())
            .collect::<Result<Vec<_>, _>>()
    }

    async fn create(&self, gift: Gift) -> Result<(), ServiceError> {
        let mut conn = self.pool.get_conn().await?;
        conn.create(GiftRecord::from_model(gift)?).await?;

        Ok(())
    }

    async fn save(&self, gift: Gift) -> Result<(), ServiceError> {
        let mut conn = self.pool.get_conn().await?;
        conn.save(GiftRecord::from_model(gift)?).await?;

        Ok(())
    }
}

#[cfg(test)]
pub mod gift_repository_mock {
    use super::*;
    use std::sync::Mutex;

    pub struct GiftRepositoryMock {
        pub created: Arc<Mutex<Vec<Gift>>>,
        pub saved: Arc<Mutex<Vec<Gift>>>,
    }

    impl GiftRepositoryMock {
        pub fn new() -> Self {
            GiftRepositoryMock {
                created: Arc::new(Mutex::new(Vec::new())),
                saved: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    #[async_trait]
    impl IGiftRepository for GiftRepositoryMock {
        async fn find_by_id(&self, gift_id: &GiftId) -> Result<Gift, ServiceError> {
            unimplemented!()
        }

        async fn find_by_user_id_status(
            &self,
            user_id: &UserId,
            status: GiftStatus,
        ) -> Result<Vec<Gift>, ServiceError> {
            unimplemented!()
        }

        async fn create(&self, gift: Gift) -> Result<(), ServiceError> {
            self.created.lock().unwrap().push(gift);

            Ok(())
        }

        async fn save(&self, gift: Gift) -> Result<(), ServiceError> {
            self.saved.lock().unwrap().push(gift);

            Ok(())
        }
    }

    pub struct GiftRepositoryItemStub {
        pub item: Arc<Mutex<Gift>>,
        pub created: Arc<Mutex<Vec<Gift>>>,
        pub saved: Arc<Mutex<Vec<Gift>>>,
    }

    impl GiftRepositoryItemStub {
        pub fn new(item: Gift) -> Self {
            GiftRepositoryItemStub {
                item: Arc::new(Mutex::new(item)),
                created: Arc::new(Mutex::new(Vec::new())),
                saved: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    #[async_trait]
    impl IGiftRepository for GiftRepositoryItemStub {
        async fn find_by_id(&self, gift_id: &GiftId) -> Result<Gift, ServiceError> {
            Ok(self.item.lock().unwrap().clone())
        }

        async fn find_by_user_id_status(
            &self,
            user_id: &UserId,
            status: GiftStatus,
        ) -> Result<Vec<Gift>, ServiceError> {
            unimplemented!()
        }

        async fn create(&self, gift: Gift) -> Result<(), ServiceError> {
            self.created.lock().unwrap().push(gift);

            Ok(())
        }

        async fn save(&self, gift: Gift) -> Result<(), ServiceError> {
            self.saved.lock().unwrap().push(gift);

            Ok(())
        }
    }
}
