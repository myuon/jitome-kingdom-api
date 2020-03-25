use crate::domain::model::Authorization;
use crate::initializer::App;
use crate::server;
use crate::wrapper::error::ServiceError;
use std::sync::Arc;

pub struct WebContext {
    app: App,
}

impl WebContext {
    fn get_authorization(req: server::Request, ctx: Arc<WebContext>) -> Authorization {
        let r = || {
            let auth = req
                .headers()
                .get("Authorization")
                .ok_or(ServiceError::unauthorized(failure::err_msg(
                    "no Authorization header",
                )))?
                .to_str()
                .map_err(|err| {
                    ServiceError::bad_request(failure::Error::from_boxed_compat(Box::new(err)))
                })?;

            ctx.app.infras.jwt_handler.authorize(auth)
        };

        Authorization::new(r())
    }
}

pub fn handlers(app: App) -> server::App<WebContext> {
    server::App::new(WebContext { app })
        .route("/hello", http::Method::GET, api_hello)
        .route("/me", http::Method::GET, api_get_me)
        .route("/gacha/daily", http::Method::POST, api_try_daily_gacha)
        .route(
            "/gacha/daily/latest",
            http::Method::GET,
            api_get_latest_daily_gacha,
        )
        .route(
            "/gacha/daily/record",
            http::Method::GET,
            api_get_daily_gacha_record,
        )
}

async fn api_hello(
    req: server::Request,
    ps: server::Params,
    ctx: Arc<WebContext>,
) -> server::Response {
    server::response_from(Ok("hello, world!"))
}

async fn api_get_me(
    req: server::Request,
    ps: server::Params,
    ctx: Arc<WebContext>,
) -> server::Response {
    let auth = WebContext::get_authorization(req, ctx.clone());

    server::response_from(ctx.app.services.user_service.get_me(auth).await)
}

async fn api_try_daily_gacha(
    req: server::Request,
    ps: server::Params,
    ctx: Arc<WebContext>,
) -> server::Response {
    let auth = WebContext::get_authorization(req, ctx.clone());

    server::response_from(ctx.app.services.gacha_service.try_daily(auth).await)
}

async fn api_get_latest_daily_gacha(
    req: server::Request,
    ps: server::Params,
    ctx: Arc<WebContext>,
) -> server::Response {
    let auth = WebContext::get_authorization(req, ctx.clone());

    server::response_from(
        ctx.app
            .services
            .gacha_service
            .get_latest_daily_event(auth)
            .await,
    )
}

async fn api_get_daily_gacha_record(
    req: server::Request,
    ps: server::Params,
    ctx: Arc<WebContext>,
) -> server::Response {
    let auth = WebContext::get_authorization(req, ctx.clone());

    server::response_from(
        ctx.app
            .services
            .gacha_service
            .get_daily_gacha_record(auth)
            .await,
    )
}
