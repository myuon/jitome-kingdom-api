use crate::domain::interface::IPointEventRepository;
use crate::domain::model::{PointEvent, UserId};
use crate::infra::ConnPool;
use crate::unixtime::UnixTime;
use crate::wrapper::error::ServiceError;
use async_trait::async_trait;
use debil::*;
use debil_mysql::*;
use std::sync::Arc;

#[derive(Table, Accessor, Clone)]
#[sql(
    table_name = "point_event",
    sql_type = "MySQLValue",
    primary_key = "user_id"
)]
pub struct PointEventRecord {
    #[sql(size = 100)]
    user_id: String,
    current: u64,
    previous: Option<u64>,
    updated_at: i64,
}

impl PointEventRecord {
    pub fn from_model(model: PointEvent) -> Self {
        PointEventRecord {
            user_id: model.user_id.0,
            current: model.current,
            previous: model.previous,
            updated_at: model.updated_at.0,
        }
    }

    pub fn into_model(self) -> PointEvent {
        PointEvent {
            user_id: UserId(self.user_id),
            current: self.current,
            previous: self.previous,
            updated_at: UnixTime(self.updated_at),
        }
    }
}

pub struct PointEventRepository {
    pool: Arc<ConnPool>,
}

impl PointEventRepository {
    pub fn new(pool: Arc<ConnPool>) -> Self {
        PointEventRepository { pool }
    }
}

#[async_trait]
impl IPointEventRepository for PointEventRepository {
    async fn find_by_id(&self, user_id: &UserId) -> Result<PointEvent, ServiceError> {
        let mut conn = self.pool.get_conn().await?;
        let record = conn
            .first_with::<PointEventRecord>(QueryBuilder::new().filter(format!(
                "{} = '{}'",
                accessor!(PointEventRecord::user_id),
                user_id.0
            )))
            .await?;

        Ok(record.into_model())
    }

    async fn save(&self, event: PointEvent) -> Result<(), ServiceError> {
        let mut conn = self.pool.get_conn().await?;
        conn.save(PointEventRecord::from_model(event)).await?;

        Ok(())
    }
}
