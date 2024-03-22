use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use serde_json::json;

#[derive(Debug, Clone)]
pub struct JsonData<T> {
    pub status: StatusCode,
    pub data: T,
    pub message: String,
}

impl<T> JsonData<T> {
    pub fn ok(data: T) -> Result<Self> {
        Ok(Self {
            status: StatusCode::OK,
            data,
            message: "".to_string(),
        })
    }

    pub fn created(data: T) -> Result<Self> {
        Ok(Self {
            status: StatusCode::CREATED,
            data,
            message: "".to_string(),
        })
    }
}

impl JsonData<serde_json::Value> {
    pub fn empty() -> Result<Self> {
        Ok(Self {
            status: StatusCode::OK,
            data: json!({}),
            message: "".to_string(),
        })
    }
}

impl<T> IntoResponse for JsonData<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        let json = json!({
            "status": self.status.as_u16(),
            "data": self.data,
            "message": self.message
        });
        (self.status, Json(json)).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, Clone)]
pub enum AppError {
    ServerError,
    ClientError { status: StatusCode, message: String },
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            Self::ServerError => {
                let data = JsonData {
                    status: StatusCode::INTERNAL_SERVER_ERROR,
                    data: serde_json::json!({}),
                    message: "INTERNAL SERVER ERROR".to_string(),
                };
                data.into_response()
            }
            Self::ClientError { status, message } => {
                let data = JsonData {
                    status,
                    data: serde_json::json!({}),
                    message,
                };
                data.into_response()
            }
        }
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(_value: E) -> Self {
        Self::ServerError
    }
}

#[derive(Debug, Clone)]
pub struct NotFound;

impl From<NotFound> for AppError {
    fn from(_value: NotFound) -> Self {
        AppError::ClientError {
            status: StatusCode::NOT_FOUND,
            message: "NOT FOUND".to_string(),
        }
    }
}
