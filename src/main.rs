#[macro_use]
extern crate log;

mod initializer;
mod web;

mod wrapper;
pub use wrapper::*;

#[tokio::main]
async fn main() {
    env_logger::init();

    let app = initializer::new();

    server::HttpServer::new()
        .bind(([0, 0, 0, 0], 1234).into())
        .service(web::handlers(app))
        .run()
        .await
        .unwrap();
}
