use crate::domain::interface::IGiftRepository;
use crate::domain::model::{Gift, GiftId, GiftStatus, GiftType, JankenEventId, UserId};
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

#[derive(Table, Clone, Accessor, Debug)]
#[sql(table_name = "gift", sql_type = "MySQLValue", primary_key = "id")]
pub struct GiftRecord {
    #[sql(size = 100)]
    pub id: String,
    pub gift_type: String,
    pub description: String,
    pub created_at: i64,
    pub janken_win_event: Option<String>,
    pub janken_lose_event: Option<String>,
}

impl GiftRecord {
    pub fn from_model(model: Gift) -> Result<Self, ServiceError> {
        Ok(GiftRecord {
            id: model.id.0.clone(),
            gift_type: serde_json::to_string::<GiftTypeRecord>(&GiftTypeRecord::from_model(
                model.gift_type,
            ))?,
            description: model.description,
            created_at: model.created_at.0,
            janken_win_event: model.janken_win_event.map(|v| v.0),
            janken_lose_event: model.janken_lose_event.map(|v| v.0),
        })
    }
}

#[derive(Table, Clone, Accessor, Debug)]
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

#[derive(Debug)]
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
    pub fn from_model(model: Gift, user_id: UserId) -> Result<Self, ServiceError> {
        Ok(JoinedGiftRecordUserRelationView {
            gift: GiftRecord::from_model(model.clone())?,
            user_relation: GiftUserRelation {
                id: model.id.0,
                user_id: user_id.0,
                status: model.status.to_string(),
            },
        })
    }

    pub fn into_model(self) -> Result<Gift, ServiceError> {
        Ok(Gift {
            id: GiftId(self.gift.id),
            gift_type: serde_json::from_str::<GiftTypeRecord>(&self.gift.gift_type)?.into_model(),
            description: self.gift.description,
            created_at: UnixTime(self.gift.created_at),
            status: GiftStatus::from_str(&self.user_relation.status),
            janken_win_event: self.gift.janken_win_event.map(|v| JankenEventId(v)),
            janken_lose_event: self.gift.janken_lose_event.map(|v| JankenEventId(v)),
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
                "{} = '{}'",
                accessor!(GiftRecord::id),
                gift_id.0
            )))
            .await?;
        let user_relation = conn
            .first_with::<GiftUserRelation>(debil::QueryBuilder::new().filter(format!(
                "{} = '{}' and {} = '{}'",
                accessor!(GiftUserRelation::id),
                gift_id.0,
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
                        "{} = '{}' AND {} = '{}'",
                        accessor!(GiftUserRelation::user_id),
                        user_id.0,
                        accessor!(GiftUserRelation::status),
                        status.to_string()
                    ))
                    .append_selects(vec![
                        accessor!(GiftUserRelation::user_id),
                        accessor!(GiftUserRelation::status),
                    ])
                    .order_by(accessor!(GiftRecord::created_at), Ordering::Descending),
            )
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

    async fn save_status(
        &self,
        gift_id: GiftId,
        user_id: UserId,
        status: GiftStatus,
    ) -> Result<(), ServiceError> {
        let mut conn = self.pool.get_conn().await?;
        conn.save(GiftUserRelation {
            id: gift_id.0,
            user_id: user_id.0,
            status: status.to_string(),
        })
        .await?;

        Ok(())
    }

    async fn create_for(
        &self,
        gift: Gift,
        users: Vec<UserId>,
        status: GiftStatus,
    ) -> Result<(), ServiceError> {
        let mut conn = self.pool.get_conn().await?;
        conn.start_transaction().await?;

        let gift_id = gift.id.clone();
        conn.create(GiftRecord::from_model(gift)?).await?;
        for user_id in users {
            conn.save(GiftUserRelation {
                id: gift_id.0.clone(),
                user_id: user_id.0,
                status: status.to_string(),
            })
            .await?;
        }
        conn.commit().await?;

        Ok(())
    }
}

#[cfg(test)]
pub mod gift_repository_mock {
    use super::*;
    use std::sync::Mutex;

    pub struct GiftRepositoryMock {
        pub created: Arc<Mutex<Vec<Gift>>>,
        pub saved: Arc<Mutex<Vec<(GiftId, UserId, GiftStatus)>>>,
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

        async fn save_status(
            &self,
            gift_id: GiftId,
            user_id: UserId,
            status: GiftStatus,
        ) -> Result<(), ServiceError> {
            self.saved.lock().unwrap().push((gift_id, user_id, status));

            Ok(())
        }
    }

    pub struct GiftRepositoryItemStub {
        pub item: Arc<Mutex<Gift>>,
        pub created: Arc<Mutex<Vec<Gift>>>,
        pub saved: Arc<Mutex<Vec<(GiftId, UserId, GiftStatus)>>>,
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
        async fn find_by_id(
            &self,
            gift_id: &GiftId,
            user_id: &UserId,
        ) -> Result<Gift, ServiceError> {
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

        async fn save_status(
            &self,
            gift_id: GiftId,
            user_id: UserId,
            status: GiftStatus,
        ) -> Result<(), ServiceError> {
            self.saved.lock().unwrap().push((gift_id, user_id, status));

            Ok(())
        }
    }
}
