use crate::wrapper::error::ServiceError;
use debil::SQLTable;
use debil_dynamodb::{into_item, DynamoType};
use rusoto_dynamodb::DynamoDb;

#[derive(Clone)]
pub struct DynamoClient {
    client: rusoto_dynamodb::DynamoDbClient,
}

#[derive(PartialEq)]
pub enum ScanOrder {
    Ascending,
    Descending,
}

impl Default for ScanOrder {
    fn default() -> Self {
        ScanOrder::Ascending
    }
}

#[derive(Default)]
pub struct QueryInput {
    pub table_name: String,
    pub index_name: Option<String>,
    pub pk_name: String,
    pub pk_value: rusoto_dynamodb::AttributeValue,
    pub limit: Option<i64>,
    pub scan_order: Option<ScanOrder>,
}

impl DynamoClient {
    pub fn new(region: rusoto_core::Region) -> DynamoClient {
        DynamoClient {
            client: rusoto_dynamodb::DynamoDbClient::new(region),
        }
    }

    pub async fn query_by_pk<T: SQLTable<ValueType = DynamoType>>(
        &self,
        input: QueryInput,
    ) -> Result<Vec<T>, ServiceError> {
        let client = self.client.clone();

        let body = client
            .query(rusoto_dynamodb::QueryInput {
                table_name: input.table_name,
                index_name: input.index_name,
                key_condition_expression: Some("#pk = :pk".to_string()),
                expression_attribute_names: Some(maplit::hashmap! {
                    "#pk".to_string() => input.pk_name,
                }),
                expression_attribute_values: Some(maplit::hashmap! {
                    ":pk".to_string() => input.pk_value,
                }),
                limit: input.limit,
                scan_index_forward: input.scan_order.map(|s| s == ScanOrder::Ascending),
                ..Default::default()
            })
            .await?;

        let items = body.items.ok_or(ServiceError::not_found(failure::err_msg(
            "record not found",
        )))?;

        Ok(debil_dynamodb::from_items(items))
    }

    pub async fn create<T: SQLTable<ValueType = DynamoType> + Send + 'static>(
        &self,
        table_name: String,
        item: T,
        pk_name: String,
    ) -> Result<(), ServiceError> {
        let client = self.client.clone();

        client
            .put_item(rusoto_dynamodb::PutItemInput {
                table_name,
                item: into_item(item),
                condition_expression: Some(format!("attribute_not_exists({})", pk_name)),
                ..Default::default()
            })
            .await?;

        Ok(())
    }

    pub async fn save<T: SQLTable<ValueType = DynamoType> + Send + 'static>(
        &self,
        table_name: String,
        item: T,
    ) -> Result<(), ServiceError> {
        let client = self.client.clone();

        client
            .put_item(rusoto_dynamodb::PutItemInput {
                table_name,
                item: into_item(item),
                ..Default::default()
            })
            .await?;

        Ok(())
    }
}
