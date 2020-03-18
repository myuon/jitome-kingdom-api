use crate::domain::service::UserService;
use crate::infra::UserRepository;
use std::sync::Arc;

pub struct Config {
    pub db_url: String,
}

pub struct Services {
    user_service: UserService,
}

pub struct App {
    services: Services,
}

pub fn new(config: Config) -> App {
    let services = Services {
        user_service: UserService::new(Arc::new(UserRepository::new())),
    };

    App { services }
}
