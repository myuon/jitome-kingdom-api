#[macro_use]
extern crate log;

mod domain;
mod infra;
mod initializer;
mod web;

mod wrapper;
pub use wrapper::*;

use crate::infra::{
    GiftRecord, GiftUserRelation, JWTHandler, JankenEventRecord, PointEventRecord, UserRecord,
};
use debil_mysql::DebilConn;
use std::env;
use std::sync::Arc;

async fn migrate(mut conn: DebilConn) -> Result<(), debil_mysql::Error> {
    conn.migrate::<UserRecord>().await?;
    conn.migrate::<GiftRecord>().await?;
    conn.migrate::<GiftUserRelation>().await?;
    conn.migrate::<JankenEventRecord>().await?;
    conn.migrate::<PointEventRecord>().await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    env::set_var("RUST_LOG", "info");
    env_logger::init();
    dotenv::dotenv().expect("Failed to load dotenv");

    let db_url = env::var("DB_URL").unwrap();
    let public_key = JWTHandler::load_from_jwk(&env::var("JWK_URL").unwrap()).await;
    let gacha_event_repository_table_name = env::var("GACHA_EVENT_REPOSITORY_TABLE_NAME").unwrap();
    let user_icon_upload_bucket = env::var("USER_ICON_UPLOAD_BUCKET").unwrap();
    let exec_task = env::var("EXECUTION_TASK");

    let app = initializer::new(initializer::Config {
        aws_region: rusoto_core::Region::ApNortheast1,
        db_url: db_url.clone(),
        public_key: Arc::new(public_key),
        gacha_event_repository_table_name,
        user_icon_upload_bucket,
    });

    match exec_task {
        Ok(task) => match task.as_str() {
            "janken" => {
                if let Err(err) = app.services.janken_process_service.run().await {
                    panic!("{:?}", err);
                }
            }
            "ranking" => {
                if let Err(err) = app.services.point_process_service.run().await {
                    panic!("{:?}", err);
                }
            }
            _ => panic!("Unsupported task: {}", task),
        },
        Err(_) => {
            let mut conn = debil_mysql::DebilConn::from_conn(
                mysql_async::Conn::from_url(db_url.clone()).await.unwrap(),
            );
            migrate(conn).await.expect("Error in migration");

            server::HttpServer::new()
                .bind(([0, 0, 0, 0], 1234).into())
                .service(web::handlers(app))
                .run()
                .await
                .unwrap();
        }
    }
}
