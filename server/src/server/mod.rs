mod layer;
use std::{net::TcpListener, sync::Arc};

use axum::extract::Extension;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::{api::register_routes, context::RawSDBApiContext};

pub type HttpApiServer =
    axum::Server<hyper::server::conn::AddrIncoming, axum::routing::IntoMakeService<axum::Router>>;

pub async fn build_http_server<T>(db_path: String) -> anyhow::Result<HttpApiServer>
where
    T: struct_db::SDBItem + Send + Sync + 'static + Serialize + Deserialize<'static>,
{
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let default_port = std::env::var("PORT").unwrap_or_else(|_| 8080.to_string());
    let default_host = "0.0.0.0";
    let default_http_addr = [default_host, &default_port].join(":");
    let db_path = std::path::Path::new(&db_path);
    let mut db = struct_db::Db::init(db_path).unwrap();
    db.define::<T>();
    let ctx = Arc::new(Mutex::new(RawSDBApiContext { db }));

    let routes = register_routes::<T, RawSDBApiContext>();
    let app = routes
        .layer(Extension(ctx))
        .layer(Extension(layer::HttpLoggerLayer::new()));

    let listener = TcpListener::bind(default_http_addr).unwrap();

    let httpserver = axum::Server::from_tcp(listener).unwrap();
    let httpserver = httpserver.serve(app.into_make_service());

    Ok(httpserver)
}
