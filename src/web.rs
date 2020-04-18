use crate::domain::model::{Authorization, GiftId, GiftStatus};
use crate::initializer::App;
use crate::server;
use crate::wrapper::error::ServiceError;
use serde::de::DeserializeOwned;
use std::sync::Arc;

pub struct WebContext {
    app: App,
}

impl WebContext {
    fn get_authorization(req: &server::Request, ctx: Arc<WebContext>) -> Authorization {
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

    async fn read_body<T: DeserializeOwned>(body: hyper::Body) -> Result<T, ServiceError> {
        use bytes::buf::BufExt;

        let body = hyper::body::aggregate(body).await.map_err(|err| {
            ServiceError::bad_request(failure::Error::from_boxed_compat(Box::new(err)))
        })?;
        let resp = serde_json::from_reader(body.reader())?;
        Ok(resp)
    }
}

pub fn handlers(app: App) -> server::App<WebContext> {
    server::App::new(WebContext { app })
        .route("/hello", http::Method::GET, api_hello)
        .route("/me", http::Method::GET, api_get_me)
        .route("/me", http::Method::PUT, api_update_me)
        .route("/me/icon", http::Method::POST, api_upload_icon)
        .route(
            "/users/:screen_name/available",
            http::Method::GET,
            api_check_user_available,
        )
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
        .route("/gift/ready", http::Method::GET, api_list_gifts_ready)
        .route("/gift/opened", http::Method::GET, api_list_gifts_opened)
        .route("/gift/:gift_id/open", http::Method::POST, api_open_gift)
        .route(
            "/admin/gift/distribute_all",
            http::Method::POST,
            api_admin_distribute_gift,
        )
        .route("/janken", http::Method::POST, api_create_janken)
        .route("/janken", http::Method::GET, api_list_janken_events)
        .route("/ranking/top", http::Method::GET, api_ranking_top)
        .route("/ranking/diff", http::Method::GET, api_ranking_diff)
}

async fn api_hello(
    req: server::Request,
    ps: server::Params,
    ctx: Arc<WebContext>,
) -> server::Response {
    server::response_from(Ok(serde_json::json!({
        "data": "hello, world!",
        "uri": req.uri().to_string()
    })))
}

async fn api_get_me(
    req: server::Request,
    ps: server::Params,
    ctx: Arc<WebContext>,
) -> server::Response {
    let auth = WebContext::get_authorization(&req, ctx.clone());

    server::response_from(ctx.app.services.user_me_service.get_me(auth).await)
}

async fn api_update_me(
    req: server::Request,
    ps: server::Params,
    ctx: Arc<WebContext>,
) -> server::Response {
    let auth = WebContext::get_authorization(&req, ctx.clone());

    server::response_from_async(async {
        let body = WebContext::read_body(req.into_body()).await?;

        ctx.app.services.user_me_service.update_me(auth, body).await
    })
    .await
}

async fn api_upload_icon(
    req: server::Request,
    ps: server::Params,
    ctx: Arc<WebContext>,
) -> server::Response {
    let auth = WebContext::get_authorization(&req, ctx.clone());

    server::response_from_async(async {
        let body = WebContext::read_body(req.into_body()).await?;

        ctx.app
            .services
            .user_icon_upload_service
            .upload(auth, body)
            .await
    })
    .await
}

async fn api_check_user_available(
    req: server::Request,
    ps: server::Params,
    ctx: Arc<WebContext>,
) -> server::Response {
    let auth = WebContext::get_authorization(&req, ctx.clone());
    let screen_name = match ps.find("screen_name") {
        None => {
            return server::response_from::<()>(Err(ServiceError::bad_request(failure::err_msg(
                "not_found",
            ))))
        }
        Some(v) => v,
    };

    server::response_from_async(async {
        ctx.app
            .services
            .user_service
            .is_screen_name_available(auth, screen_name)
            .await
    })
    .await
}

async fn api_try_daily_gacha(
    req: server::Request,
    ps: server::Params,
    ctx: Arc<WebContext>,
) -> server::Response {
    let auth = WebContext::get_authorization(&req, ctx.clone());

    server::response_from(ctx.app.services.gacha_service.try_daily(auth).await)
}

async fn api_get_latest_daily_gacha(
    req: server::Request,
    ps: server::Params,
    ctx: Arc<WebContext>,
) -> server::Response {
    let auth = WebContext::get_authorization(&req, ctx.clone());

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
    let auth = WebContext::get_authorization(&req, ctx.clone());

    server::response_from(
        ctx.app
            .services
            .gacha_service
            .get_daily_gacha_record(auth)
            .await,
    )
}

async fn api_list_gifts_ready(
    req: server::Request,
    ps: server::Params,
    ctx: Arc<WebContext>,
) -> server::Response {
    let auth = WebContext::get_authorization(&req, ctx.clone());

    server::response_from(
        ctx.app
            .services
            .gift_service
            .list_by_status(auth, GiftStatus::Ready)
            .await,
    )
}

async fn api_list_gifts_opened(
    req: server::Request,
    ps: server::Params,
    ctx: Arc<WebContext>,
) -> server::Response {
    let auth = WebContext::get_authorization(&req, ctx.clone());

    server::response_from(
        ctx.app
            .services
            .gift_service
            .list_by_status(auth, GiftStatus::Opened)
            .await,
    )
}

async fn api_open_gift(
    req: server::Request,
    ps: server::Params,
    ctx: Arc<WebContext>,
) -> server::Response {
    let auth = WebContext::get_authorization(&req, ctx.clone());
    let gift_id = match ps.find("gift_id") {
        None => {
            return server::response_from::<()>(Err(ServiceError::bad_request(failure::err_msg(
                "not_found",
            ))))
        }
        Some(v) => v,
    };

    server::response_from(
        ctx.app
            .services
            .gift_service
            .open(auth, &GiftId(gift_id))
            .await,
    )
}

async fn api_admin_distribute_gift(
    req: server::Request,
    ps: server::Params,
    ctx: Arc<WebContext>,
) -> server::Response {
    let auth = WebContext::get_authorization(&req, ctx.clone());

    server::response_from_async(async {
        let body = WebContext::read_body(req.into_body()).await?;

        ctx.app
            .services
            .gift_distribution_service
            .distribute_point(auth, body)
            .await
    })
    .await
}

async fn api_create_janken(
    req: server::Request,
    ps: server::Params,
    ctx: Arc<WebContext>,
) -> server::Response {
    let auth = WebContext::get_authorization(&req, ctx.clone());

    server::response_from_async(async {
        let body = WebContext::read_body(req.into_body()).await?;

        ctx.app.services.janken_service.create(auth, body).await
    })
    .await
}

async fn api_list_janken_events(
    req: server::Request,
    ps: server::Params,
    ctx: Arc<WebContext>,
) -> server::Response {
    let auth = WebContext::get_authorization(&req, ctx.clone());
    // urlが相対パスをパースできないので適当にoriginを設定
    let query = url::Url::parse("http://localhost")
        .and_then(|u| u.join(&req.uri().to_string()))
        .ok()
        .and_then(|u| {
            u.query_pairs()
                .into_iter()
                .collect::<std::collections::HashMap<_, _>>()
                .get("limit")
                .and_then(|r| r.parse::<i32>().ok())
        })
        .unwrap_or(20);

    server::response_from_async(ctx.app.services.janken_service.find_by_user_id(auth, query)).await
}

async fn api_ranking_top(
    req: server::Request,
    ps: server::Params,
    ctx: Arc<WebContext>,
) -> server::Response {
    let auth = WebContext::get_authorization(&req, ctx.clone());

    server::response_from(
        ctx.app
            .services
            .point_ranking_service
            .list_by_points(auth)
            .await,
    )
}

async fn api_ranking_diff(
    req: server::Request,
    ps: server::Params,
    ctx: Arc<WebContext>,
) -> server::Response {
    let auth = WebContext::get_authorization(&req, ctx.clone());

    server::response_from(
        ctx.app
            .services
            .point_ranking_service
            .list_by_diff(auth)
            .await,
    )
}
