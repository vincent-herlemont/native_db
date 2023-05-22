use crate::context::{Result, SDBApiContext};
use axum::extract::{self, Extension};
use axum::response::IntoResponse;
use axum::routing::get as Get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use struct_db::SDBItem;
use tokio::sync::Mutex;

pub fn register_routes<T, H>() -> Router
where
    T: SDBItem + Serialize + Deserialize<'static> + Send + Sync + 'static,
    H: SDBApiContext<T>,
{
    let table_name = T::struct_db_schema().table_name;
    Router::new().route(&format!("/{}/:id", table_name), Get(get::<T, H>))
}

pub async fn get<T, H>(
    Extension(ctx): extract::Extension<Arc<Mutex<H>>>,
    extract::Path(id): extract::Path<i32>,
) -> Result<impl IntoResponse>
where
    T: SDBItem + Serialize + Send + Sync + 'static,
    H: SDBApiContext<T>,
{
    let ctx = ctx.lock().await;
    let res = ctx.get(&id.to_be_bytes()).await?;
    Ok(Json(res))
}
