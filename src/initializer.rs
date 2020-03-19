use crate::domain::service::UserService;
use crate::infra::{ConnPool, JWTHandler, UserRepository};
use std::sync::Arc;

pub struct Config {
    pub db_url: String,
    pub public_key: Arc<biscuit::jwk::JWKSet<biscuit::Empty>>,
}

pub struct Infras {
    jwt_handler: Arc<JWTHandler>,
    user_repository: Arc<UserRepository>,
}

pub struct Services {
    user_service: UserService,
}

pub struct App {
    infras: Infras,
    services: Services,
}

pub fn new(config: Config) -> App {
    let conn_pool = Arc::new(ConnPool::new(&config.db_url).unwrap());
    let infras = Infras {
        jwt_handler: Arc::new(JWTHandler::new(config.public_key)),
        user_repository: Arc::new(UserRepository::new(conn_pool.clone())),
    };
    let services = Services {
        user_service: UserService::new(infras.user_repository.clone()),
    };

    App { infras, services }
}
