use crate::initializer::App;
use crate::server;
use std::sync::Arc;

pub struct WebContext {
    app: App,
}

pub fn handlers(app: App) -> server::App<WebContext> {
    server::App::new(WebContext { app }).route("/hello", http::Method::GET, api_hello)
}

async fn api_hello(
    req: server::Request,
    ps: server::Params,
    ctx: Arc<WebContext>,
) -> server::Response {
    server::response_from(Ok("hello, world!"))
}
