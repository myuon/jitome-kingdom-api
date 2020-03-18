#[macro_use]
extern crate log;

mod domain;
mod infra;
mod initializer;
mod web;

mod wrapper;
pub use wrapper::*;

use std::env;

#[tokio::main]
async fn main() {
    env_logger::init();
    dotenv::dotenv().expect("Failed to load dotenv");

    let db_url = env::var("DB_URL").unwrap();

    let app = initializer::new(initializer::Config { db_url });

    server::HttpServer::new()
        .bind(([0, 0, 0, 0], 1234).into())
        .service(web::handlers(app))
        .run()
        .await
        .unwrap();
}
