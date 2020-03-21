use crate::domain::service::{GachaService, UserService};
use crate::infra::{ConnPool, DynamoClient, GachaEventRepository, JWTHandler, UserRepository};
use std::sync::Arc;

pub struct Config {
    pub aws_region: rusoto_core::Region,
    pub db_url: String,
    pub public_key: Arc<biscuit::jwk::JWKSet<biscuit::Empty>>,
    pub gacha_event_repository_table_name: String,
}

pub struct Infras {
    pub dynamo_client: Arc<DynamoClient>,
    pub jwt_handler: Arc<JWTHandler>,
    pub user_repository: Arc<UserRepository>,
    pub gacha_event_repository: Arc<GachaEventRepository>,
}

pub struct Services {
    pub user_service: UserService,
    pub gacha_service: GachaService,
}

pub struct App {
    pub infras: Infras,
    pub services: Services,
}

pub fn new(config: Config) -> App {
    let conn_pool = Arc::new(ConnPool::new(&config.db_url).unwrap());
    let dynamo_client = Arc::new(DynamoClient::new(config.aws_region));
    let infras = Infras {
        dynamo_client: dynamo_client.clone(),
        jwt_handler: Arc::new(JWTHandler::new(config.public_key)),
        user_repository: Arc::new(UserRepository::new(conn_pool.clone())),
        gacha_event_repository: Arc::new(GachaEventRepository::new(
            dynamo_client.clone(),
            config.gacha_event_repository_table_name,
        )),
    };
    let services = Services {
        user_service: UserService::new(infras.user_repository.clone()),
        gacha_service: GachaService::new(
            infras.gacha_event_repository.clone(),
            infras.user_repository.clone(),
        ),
    };

    App { infras, services }
}
