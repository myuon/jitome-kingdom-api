use crate::error::ServiceError;
use futures::prelude::*;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Server};
use path_tree::PathTree;
use std::pin::Pin;
use std::sync::Arc;

pub type Request = hyper::Request<hyper::Body>;
pub type Response = hyper::Response<hyper::Body>;

pub async fn response_from_async<D: serde::Serialize>(
    result: impl Future<Output = Result<D, ServiceError>>,
) -> Response {
    response_from(result.await)
}

pub fn response_from<D: serde::Serialize>(result: Result<D, ServiceError>) -> Response {
    let (s, b) = match result {
        Ok(d) => (
            hyper::StatusCode::OK,
            hyper::Body::from(serde_json::to_string(&d).unwrap()),
        ),
        Err(err) => {
            error!("{:?}", err);

            (err.status_code, hyper::Body::from(err.error.to_string()))
        }
    };

    hyper::Response::builder()
        .status(s)
        .header(hyper::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .header(hyper::header::CONTENT_TYPE, "application/json")
        .body(b)
        .unwrap()
}

pub struct Params(Vec<(String, String)>);

impl Params {
    pub fn new(vec: Vec<(String, String)>) -> Params {
        Params(vec)
    }

    pub fn find(&self, name: impl Into<String>) -> Option<String> {
        let key = name.into();
        self.0
            .iter()
            .find(|(x, _)| x == &key)
            .map(|(_, y)| y)
            .cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_find_params() {
        let params = Params(vec![
            ("a".to_string(), "b".to_string()),
            ("x".to_string(), "y".to_string()),
        ]);

        assert_eq!(params.find("a"), Some("b".to_string()));
        assert_eq!(params.find("x"), Some("y".to_string()));
        assert_eq!(params.find("ttt"), None);
    }
}

type FutureResult<O> = Pin<Box<dyn Future<Output = O> + Send>>;
type Handler<D> = Arc<dyn Fn(Request, Params, Arc<D>) -> FutureResult<Response> + Sync + Send>;

pub struct App<D> {
    paths: PathTree<Handler<D>>,
    data: Arc<D>,
}

impl<D> Clone for App<D> {
    fn clone(&self) -> Self {
        App {
            paths: self.paths.clone(),
            data: self.data.clone(),
        }
    }
}

fn internal_path(method: &Method, path: &str) -> String {
    format!("/{}/{}", method, path)
}

async fn cors_handler<D>(_req: Request, _params: Params, _data: Arc<D>) -> Response {
    hyper::Response::builder()
        .header(hyper::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        // Safari対策に全部明記する
        .header(
            hyper::header::ACCESS_CONTROL_ALLOW_HEADERS,
            "*, Authorization, Content-Type, Origin, Referer, Accept, User-Agent",
        )
        .header(
            hyper::header::ACCESS_CONTROL_ALLOW_METHODS,
            "POST,PUT,DELETE",
        )
        .body(Body::default())
        .unwrap()
}

impl<D: Sync + Send + 'static> App<D> {
    pub fn new(data: D) -> App<D> {
        App {
            paths: PathTree::new(),
            data: Arc::new(data),
        }
    }

    pub fn route<F, T>(mut self, path: &str, method: Method, f: F) -> Self
    where
        F: Fn(Request, Params, Arc<D>) -> T + Clone + Sync + Send + 'static,
        T: Future<Output = Response> + Send + 'static,
    {
        let ipath = internal_path(&method, path);
        if self.paths.find(&ipath).is_some() {
            error!("The path {:?} does already exist.", ipath);
        }

        self.paths
            .insert(&ipath, Arc::new(move |r, p, d| Box::pin(f(r, p, d))));

        // CORS (こんな適当でいいのか？)
        self.paths.insert(
            internal_path(&http::Method::OPTIONS, path).as_str(),
            Arc::new(move |r, p, d| Box::pin(cors_handler(r, p, d))),
        );

        self
    }
}

pub struct HttpServer<D> {
    addr: Option<std::net::SocketAddr>,
    app: Option<App<D>>,
}

impl<D: Sync + Send + 'static> HttpServer<D> {
    pub fn new() -> HttpServer<D> {
        HttpServer {
            addr: None,
            app: None,
        }
    }

    pub fn bind(&mut self, addr: std::net::SocketAddr) -> &mut Self {
        self.addr = Some(addr);

        self
    }

    pub fn service(&mut self, app: App<D>) -> &mut Self {
        self.app = Some(app);

        self
    }

    pub async fn run(&mut self) -> Result<(), hyper::Error> {
        let addr = self.addr.take().unwrap();
        let app = self.app.take().unwrap();
        let server = Server::bind(&addr).serve(make_service_fn(|_| {
            let app = app.clone();

            async {
                Ok::<_, hyper::Error>(service_fn(move |req| {
                    let paths = app.paths.clone();
                    let p = internal_path(req.method(), req.uri().path());
                    let data = app.data.clone();

                    async move {
                        match paths.find(p.as_str()) {
                            None => hyper::Response::builder().status(404).body(Body::from("")),
                            Some((f, ps)) => {
                                let result = f(
                                    req,
                                    Params(
                                        ps.iter()
                                            .map(|(x, y)| (x.to_string(), y.to_string()))
                                            .collect::<Vec<_>>(),
                                    ),
                                    data.clone(),
                                )
                                .await;

                                Ok(result)
                            }
                        }
                    }
                }))
            }
        }));

        println!("Listening on http://{}", addr);
        server.await
    }
}
