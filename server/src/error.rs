use std::fmt::{Display, Formatter, Result};

use axum::response::IntoResponse;
use axum::{http::StatusCode, response::Response};
use serde::{Serialize, Serializer};

#[derive(Debug, Serialize, thiserror::Error)]
pub struct HttpErrResp {
    #[serde(serialize_with = "serialize_statuscode")]
    pub code: StatusCode,
    pub msg: String,
}

fn serialize_statuscode<S>(x: &StatusCode, s: S) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_u16(x.as_u16())
}

impl IntoResponse for HttpErrResp {
    fn into_response(self) -> axum::response::Response {
        let payload = serde_json::to_string(&self).unwrap();
        let body = axum::body::boxed(axum::body::Full::from(payload));
        Response::builder().status(self.code).body(body).unwrap()
    }
}

impl Display for HttpErrResp {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "[{}]: {}", self.code, self.msg)
    }
}

impl HttpErrResp {
    pub fn not_found(msg: &str) -> Self {
        Self {
            code: StatusCode::NOT_FOUND,
            msg: msg.to_string(),
        }
    }

    pub fn internal_server_err(msg: &str) -> Self {
        Self {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            msg: msg.to_string(),
        }
    }
}
