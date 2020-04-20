use crate::domain::interface::IRankingRepository;
use crate::domain::model::{PointDiffRankingRecord, PointRankingRecord};
use crate::infra::{ConnPool, PointEventRecord, UserRecord};
use crate::wrapper::error::ServiceError;
use async_trait::async_trait;
use debil::*;
use debil_mysql::*;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::sync::Arc;

pub struct RankingRepository {
    pool: Arc<ConnPool>,
}

impl RankingRepository {
    pub fn new(pool: Arc<ConnPool>) -> Self {
        RankingRepository { pool }
    }
}

struct JoinedRankingView {
    user: UserRecord,
    diff: Option<u64>,
}

impl SQLMapper for JoinedRankingView {
    type ValueType = MySQLValue;

    fn map_from_sql(hm: HashMap<String, Self::ValueType, RandomState>) -> Self {
        JoinedRankingView {
            diff: hm["diff"].clone().deserialize(),
            user: map_from_sql(hm),
        }
    }
}

#[async_trait]
impl IRankingRepository for RankingRepository {
    async fn list_top_points(&self, limit: u64) -> Result<Vec<PointRankingRecord>, ServiceError> {
        let mut conn = self.pool.get_conn().await?;
        let users = conn
            .load_with::<UserRecord>(
                QueryBuilder::new()
                    .order_by(accessor!(UserRecord::point), Ordering::Descending)
                    .limit(limit as i32),
            )
            .await?;

        Ok(users
            .into_iter()
            .map(|user| PointRankingRecord::new(user.into_model()))
            .collect())
    }

    async fn list_top_point_diffs(
        &self,
        limit: u64,
    ) -> Result<Vec<PointDiffRankingRecord>, ServiceError> {
        let mut conn = self.pool.get_conn().await?;

        let views = conn
            .load_with2::<PointEventRecord, JoinedRankingView>(
                // FIXME: accessor macro
                QueryBuilder::new()
                    .inner_join(
                        table_name::<UserRecord>(),
                        (
                            accessor_name!(PointEventRecord::user_id),
                            accessor_name!(UserRecord::id),
                        ),
                    )
                    .order_by(
                        format!(
                            "({} - {})",
                            accessor!(PointEventRecord::current),
                            accessor!(PointEventRecord::previous)
                        ),
                        Ordering::Descending,
                    )
                    .append_selects(vec![
                        format!(
                            "({} - {}) AS diff",
                            accessor!(PointEventRecord::current),
                            accessor!(PointEventRecord::previous)
                        ),
                        // ↓ これないと動かないのはなぜ？
                        format!("{}.*", table_name::<UserRecord>()),
                    ])
                    .limit(limit as i32),
            )
            .await?;

        Ok(views
            .into_iter()
            .map(|view| PointDiffRankingRecord::new(view.user.into_model(), view.diff.unwrap_or(0)))
            .collect::<Vec<_>>())
    }
}
