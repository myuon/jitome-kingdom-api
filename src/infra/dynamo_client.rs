use crate::wrapper::error::ServiceError;
use dynomite::Item;
use rusoto_dynamodb::DynamoDb;

#[derive(Clone)]
pub struct DynamoClient {
    client: dynomite::dynamodb::DynamoDbClient,
}

#[derive(Default)]
pub struct QueryInput {
    pub table_name: String,
    pub index_name: Option<String>,
    pub pk_name: String,
    pub pk_value: dynomite::dynamodb::AttributeValue,
}

impl DynamoClient {
    pub fn new(region: rusoto_core::Region) -> DynamoClient {
        DynamoClient {
            client: dynomite::dynamodb::DynamoDbClient::new(region),
        }
    }

    pub async fn query_by_pk<T: Item>(&self, input: QueryInput) -> Result<Vec<T>, ServiceError> {
        let client = self.client.clone();

        let body = tokio::task::spawn_blocking(move || {
            client
                .query(dynomite::dynamodb::QueryInput {
                    table_name: input.table_name,
                    index_name: input.index_name,
                    key_condition_expression: Some("#pk = :pk".to_string()),
                    expression_attribute_names: Some(maplit::hashmap! {
                        "#pk".to_string() => input.pk_name,
                    }),
                    expression_attribute_values: Some(maplit::hashmap! {
                        ":pk".to_string() => input.pk_value,
                    }),
                    ..Default::default()
                })
                .sync()
        })
        .await??;

        let items = body.items.ok_or(ServiceError::not_found(failure::err_msg(
            "record not found",
        )))?;

        items
            .into_iter()
            .map(|item| {
                T::from_attrs(item).map_err(|err| {
                    ServiceError::internal_server_error(failure::err_msg(format!("{:?}", err)))
                })
            })
            .collect::<Result<Vec<_>, _>>()
    }

    pub async fn create<T: Item + Send + 'static>(
        &self,
        table_name: String,
        item: T,
        pk_name: String,
    ) -> Result<(), ServiceError> {
        let client = self.client.clone();

        tokio::task::spawn_blocking(move || {
            client
                .put_item(dynomite::dynamodb::PutItemInput {
                    table_name,
                    item: item.into(),
                    condition_expression: Some(format!("attribute_not_exists({})", pk_name)),
                    ..Default::default()
                })
                .sync()
        })
        .await??;

        Ok(())
    }

    pub async fn save<T: Item + Send + 'static>(
        &self,
        table_name: String,
        item: T,
    ) -> Result<(), ServiceError> {
        let client = self.client.clone();

        tokio::task::spawn_blocking(move || {
            client
                .put_item(dynomite::dynamodb::PutItemInput {
                    table_name,
                    item: item.into(),
                    ..Default::default()
                })
                .sync()
        })
        .await??;

        Ok(())
    }
}
