use crate::domain::service::{
    GachaService, GiftDistributionService, GiftService, JankenProcessService, JankenService,
    PointProcessService, UserIconUploadService, UserMeService, UserService,
};
use crate::infra::{
    ConnPool, DynamoClient, GachaEventRepository, GiftRepository, JWTHandler,
    JankenEventRepository, PointEventRepository, S3Client, UserIconUploader, UserRepository,
};
use std::sync::Arc;

pub struct Config {
    pub aws_region: rusoto_core::Region,
    pub db_url: String,
    pub public_key: Arc<biscuit::jwk::JWKSet<biscuit::Empty>>,
    pub gacha_event_repository_table_name: String,
    pub user_icon_upload_bucket: String,
}

pub struct Infras {
    pub dynamo_client: Arc<DynamoClient>,
    pub s3_client: Arc<S3Client>,
    pub jwt_handler: Arc<JWTHandler>,
    pub user_repository: Arc<UserRepository>,
    pub gacha_event_repository: Arc<GachaEventRepository>,
    pub gift_repository: Arc<GiftRepository>,
    pub user_icon_uploader: Arc<UserIconUploader>,
    pub janken_repository: Arc<JankenEventRepository>,
    pub point_repository: Arc<PointEventRepository>,
}

pub struct Services {
    pub user_me_service: UserMeService,
    pub user_service: UserService,
    pub gacha_service: GachaService,
    pub gift_service: GiftService,
    pub gift_distribution_service: GiftDistributionService,
    pub user_icon_upload_service: UserIconUploadService,
    pub janken_service: JankenService,
    pub janken_process_service: JankenProcessService,
    pub point_process_service: PointProcessService,
}

pub struct App {
    pub infras: Infras,
    pub services: Services,
}

pub fn new(config: Config) -> App {
    let conn_pool = Arc::new(ConnPool::new(&config.db_url).unwrap());
    let dynamo_client = Arc::new(DynamoClient::new(config.aws_region.clone()));
    let s3_client = Arc::new(S3Client::new(config.aws_region.clone()));
    let infras = Infras {
        dynamo_client: dynamo_client.clone(),
        s3_client: s3_client.clone(),
        jwt_handler: Arc::new(JWTHandler::new(config.public_key)),
        user_repository: Arc::new(UserRepository::new(conn_pool.clone())),
        gacha_event_repository: Arc::new(GachaEventRepository::new(
            dynamo_client.clone(),
            config.gacha_event_repository_table_name,
        )),
        gift_repository: Arc::new(GiftRepository::new(conn_pool.clone())),
        user_icon_uploader: Arc::new(UserIconUploader::new(
            s3_client.clone(),
            config.user_icon_upload_bucket,
        )),
        janken_repository: Arc::new(JankenEventRepository::new(conn_pool.clone())),
        point_repository: Arc::new(PointEventRepository::new(conn_pool.clone())),
    };
    let services = Services {
        user_me_service: UserMeService::new(infras.user_repository.clone()),
        user_service: UserService::new(infras.user_repository.clone()),
        gacha_service: GachaService::new(
            infras.gacha_event_repository.clone(),
            infras.user_repository.clone(),
        ),
        gift_service: GiftService::new(
            infras.gift_repository.clone(),
            infras.user_repository.clone(),
        ),
        gift_distribution_service: GiftDistributionService::new(
            infras.user_repository.clone(),
            infras.gift_repository.clone(),
        ),
        user_icon_upload_service: UserIconUploadService::new(
            infras.user_repository.clone(),
            infras.user_icon_uploader.clone(),
        ),
        janken_service: JankenService::new(
            infras.user_repository.clone(),
            infras.janken_repository.clone(),
        ),
        janken_process_service: JankenProcessService::new(
            infras.janken_repository.clone(),
            infras.gift_repository.clone(),
            infras.user_repository.clone(),
        ),
        point_process_service: PointProcessService::new(
            infras.user_repository.clone(),
            infras.point_repository.clone(),
        ),
    };

    App { infras, services }
}
