use crate::domain::interface::IGachaEventRepository;
use crate::domain::model::{GachaEvent, GachaEventId, GachaType, UserId};
use crate::infra::{DynamoClient, QueryInput, ScanOrder};
use crate::unixtime::UnixTime;
use crate::wrapper::error::ServiceError;
use async_trait::async_trait;
use debil::*;
use debil_dynamodb::Attribute;
use std::sync::Arc;

#[derive(Clone, Table)]
#[sql(
    table_name = "gacha_events",
    sql_type = "debil_dynamodb::DynamoType",
    primary_key = "id"
)]
pub struct GachaEventRecord {
    id: String,
    user_id: String,
    gacha_type: String,
    created_at: i64,
    gsi_user_id_gacha_type: String,
}

impl GachaEventRecord {
    pub fn into_model(self) -> GachaEvent {
        GachaEvent {
            id: GachaEventId(self.id),
            user_id: UserId(self.user_id),
            gacha_type: GachaType::new(self.gacha_type.as_str()),
            created_at: UnixTime(self.created_at),
        }
    }

    pub fn from_model(model: GachaEvent) -> Self {
        let gsi_user_id_gacha_type =
            GachaEventRecord::generate_gsi_user_id_gacha_type(&model.user_id, &model.gacha_type);

        GachaEventRecord {
            id: model.id.0,
            user_id: model.user_id.0,
            gacha_type: model.gacha_type.to_string(),
            created_at: model.created_at.0,
            gsi_user_id_gacha_type,
        }
    }

    fn generate_gsi_user_id_gacha_type(user_id: &UserId, gacha_type: &GachaType) -> String {
        format!("{}#{}", user_id.0, gacha_type.to_string())
    }
}

pub struct GachaEventRepository {
    dynamo_client: Arc<DynamoClient>,
    table_name: String,
}

impl GachaEventRepository {
    pub fn new(client: Arc<DynamoClient>, table_name: String) -> Self {
        GachaEventRepository {
            dynamo_client: client,
            table_name,
        }
    }
}

#[async_trait]
impl IGachaEventRepository for GachaEventRepository {
    async fn find_by_user_type(
        &self,
        user_id: &UserId,
        gacha_type: &GachaType,
    ) -> Result<GachaEvent, ServiceError> {
        let events = self
            .dynamo_client
            .query_by_pk::<GachaEventRecord>(QueryInput {
                table_name: self.table_name.clone(),
                index_name: Some("user_id_gacha_type".to_string()),
                pk_name: "gsi_user_id_gacha_type".to_string(),
                pk_value: GachaEventRecord::generate_gsi_user_id_gacha_type(user_id, gacha_type)
                    .into_attr(),
                scan_order: Some(ScanOrder::Descending),
                limit: Some(1 as i64),
            })
            .await?;

        if events.len() == 0 {
            return Err(ServiceError::not_found(failure::err_msg("event not found")));
        }

        Ok(events[0].clone().into_model())
    }

    async fn create(&self, event: GachaEvent) -> Result<(), ServiceError> {
        self.dynamo_client
            .create(
                self.table_name.clone(),
                GachaEventRecord::from_model(event),
                "id".to_string(),
            )
            .await?;

        Ok(())
    }
}
