use crate::error::ServiceError;
use crate::initializer::App;
use crate::server;
use std::sync::Arc;

pub struct WebContext {
    app: App,
}

pub fn handlers(app: App) -> server::App<WebContext> {
    server::App::new(WebContext { app }).route("/hello", http::Method::GET, api_hello)
}

fn response_ok<D: serde::Serialize>(d: &D) -> server::Response {
    hyper::Response::builder()
        .status(hyper::StatusCode::OK)
        .header(hyper::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .header(hyper::header::CONTENT_TYPE, "application/json")
        .body(hyper::Body::from(serde_json::to_string(d).unwrap()))
        .unwrap()
}

async fn api_hello(
    req: server::Request,
    ps: server::Params,
    ctx: Arc<WebContext>,
) -> Result<server::Response, ServiceError> {
    Ok(response_ok(&"hello, world!".to_string()))
}
