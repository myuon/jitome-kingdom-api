use crate::domain::interface::IGiftRepository;
use crate::domain::model::{Gift, GiftId, GiftStatus, GiftType, UserId};
use crate::infra::ConnPool;
use crate::wrapper::error::ServiceError;
use crate::wrapper::unixtime::UnixTime;
use async_trait::async_trait;
use debil::*;
use debil_mysql::*;
use serde::*;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub enum GiftTypeRecord {
    #[serde(rename = "point")]
    Point(u64),
}

impl GiftTypeRecord {
    pub fn from_model(model: GiftType) -> Self {
        use GiftType::*;

        match model {
            Point(p) => GiftTypeRecord::Point(p),
        }
    }

    pub fn into_model(self) -> GiftType {
        use GiftTypeRecord::*;

        match self {
            Point(p) => GiftType::Point(p),
        }
    }
}

#[derive(Table, Clone, Accessor)]
#[sql(table_name = "gift", sql_type = "MySQLValue", primary_key = "id")]
pub struct GiftRecord {
    #[sql(size = 100)]
    pub id: String,
    pub gift_type: String,
    pub description: String,
    pub created_at: i64,
}

#[derive(Table, Clone, Accessor)]
#[sql(
    table_name = "gift_user_relation",
    sql_type = "MySQLValue",
    primary_key = "id, user_id"
)]
pub struct GiftUserRelation {
    #[sql(size = 100)]
    pub id: String,
    #[sql(size = 100)]
    pub user_id: String,
    #[sql(size = 50)]
    pub status: String,
}

struct JoinedGiftRecordUserRelationView {
    gift: GiftRecord,
    user_relation: GiftUserRelation,
}

impl SQLMapper for JoinedGiftRecordUserRelationView {
    type ValueType = MySQLValue;

    fn map_from_sql(h: HashMap<String, Self::ValueType>) -> Self {
        JoinedGiftRecordUserRelationView {
            gift: map_from_sql::<GiftRecord>(h.clone()),
            user_relation: map_from_sql::<GiftUserRelation>(h),
        }
    }
}

impl JoinedGiftRecordUserRelationView {
    pub fn from_model(model: Gift) -> Result<Self, ServiceError> {
        Ok(JoinedGiftRecordUserRelationView {
            gift: GiftRecord {
                id: model.id.0.clone(),
                gift_type: serde_json::to_string::<GiftTypeRecord>(&GiftTypeRecord::from_model(
                    model.gift_type,
                ))?,
                description: model.description,
                created_at: model.created_at.0,
            },
            user_relation: GiftUserRelation {
                id: model.id.0,
                user_id: model.user_id.0,
                status: model.status.to_string(),
            },
        })
    }

    pub fn into_model(self) -> Result<Gift, ServiceError> {
        Ok(Gift {
            id: GiftId(self.gift.id),
            gift_type: serde_json::from_str::<GiftTypeRecord>(&self.gift.gift_type)?.into_model(),
            description: self.gift.description,
            user_id: UserId(self.user_relation.user_id),
            created_at: UnixTime(self.gift.created_at),
            status: GiftStatus::from_str(&self.user_relation.status),
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
    async fn find_by_id(&self, gift_id: &GiftId, user_id: &UserId) -> Result<Gift, ServiceError> {
        let mut conn = self.pool.get_conn().await?;
        let gift = conn
            .first_with::<GiftRecord>(debil::QueryBuilder::new().filter(format!(
                "{}.{} = '{}'",
                table_name::<GiftRecord>(),
                accessor!(GiftRecord::id),
                gift_id.0
            )))
            .await?;
        let user_relation = conn
            .first_with::<GiftUserRelation>(debil::QueryBuilder::new().filter(format!(
                "{}.{} = '{}' and {}.{} = '{}'",
                table_name::<GiftUserRelation>(),
                accessor!(GiftUserRelation::id),
                gift_id.0,
                table_name::<GiftUserRelation>(),
                accessor!(GiftUserRelation::user_id),
                user_id.0,
            )))
            .await?;

        JoinedGiftRecordUserRelationView {
            gift,
            user_relation,
        }
        .into_model()
    }

    async fn find_by_user_id_status(
        &self,
        user_id: &UserId,
        status: GiftStatus,
    ) -> Result<Vec<Gift>, ServiceError> {
        let mut conn = self.pool.get_conn().await?;
        let records = conn
            .load_with2::<GiftRecord, JoinedGiftRecordUserRelationView>(
                debil::QueryBuilder::new()
                    .inner_join(table_name::<GiftUserRelation>(), ("id", "id"))
                    .filter(format!(
                        "{}.{} = '{}' AND {}.{} = '{}'",
                        table_name::<GiftRecord>(),
                        accessor!(GiftUserRelation::user_id),
                        user_id.0,
                        table_name::<GiftUserRelation>(),
                        accessor!(GiftUserRelation::status),
                        status.to_string()
                    ))
                    .append_selects(vec![
                        format!(
                            "{}.{}",
                            table_name::<GiftUserRelation>(),
                            accessor!(GiftUserRelation::user_id)
                        ),
                        format!(
                            "{}.{}",
                            table_name::<GiftUserRelation>(),
                            accessor!(GiftUserRelation::status)
                        ),
                    ]),
            )
            .await?;

        records
            .into_iter()
            .map(|record| record.into_model())
            .collect::<Result<Vec<_>, _>>()
    }

    async fn create(&self, gift: Gift) -> Result<(), ServiceError> {
        let mut conn = self.pool.get_conn().await?;
        let v = JoinedGiftRecordUserRelationView::from_model(gift)?;
        conn.create(v.gift).await?;
        conn.create(v.user_relation).await?;

        Ok(())
    }

    async fn save_status(&self, gift: Gift) -> Result<(), ServiceError> {
        let mut conn = self.pool.get_conn().await?;
        let v = JoinedGiftRecordUserRelationView::from_model(gift)?;
        conn.save(v.user_relation).await?;

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
        async fn find_by_id(
            &self,
            gift_id: &GiftId,
            user_id: &UserId,
        ) -> Result<Gift, ServiceError> {
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

        async fn save_status(&self, gift: Gift) -> Result<(), ServiceError> {
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
