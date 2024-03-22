use std::ops::Deref;

use axum::{
    extract::{
        rejection::{JsonRejection, PathRejection, QueryRejection},
        FromRequest, FromRequestParts,
    },
    http::StatusCode,
    response::IntoResponse,
};

use super::response::JsonData;

#[derive(FromRequest)]
#[from_request(via(axum::extract::Json), rejection(ErrorRejection))]
pub struct Json<T>(pub T);

impl<T> Deref for Json<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(FromRequestParts)]
#[from_request(via(axum::extract::Path), rejection(ErrorRejection))]
pub struct Path<T>(pub T);

impl<T> Deref for Path<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(FromRequestParts)]
#[from_request(via(axum::extract::Query), rejection(ErrorRejection))]
pub struct Query<T>(pub T);

impl<T> Deref for Query<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct ErrorRejection {
    status: StatusCode,
    message: String,
}

impl IntoResponse for ErrorRejection {
    fn into_response(self) -> axum::response::Response {
        let data = JsonData {
            status: self.status,
            data: serde_json::json!({}),
            message: self.message,
        };
        (self.status, data).into_response()
    }
}

impl From<JsonRejection> for ErrorRejection {
    fn from(rejection: JsonRejection) -> Self {
        Self {
            status: rejection.status(),
            message: rejection.body_text(),
        }
    }
}

impl From<PathRejection> for ErrorRejection {
    fn from(rejection: PathRejection) -> Self {
        Self {
            status: rejection.status(),
            message: rejection.body_text(),
        }
    }
}

impl From<QueryRejection> for ErrorRejection {
    fn from(rejection: QueryRejection) -> Self {
        Self {
            status: rejection.status(),
            message: rejection.body_text(),
        }
    }
}
