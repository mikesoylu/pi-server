use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("session busy: {0}")]
    SessionBusy(String),
    #[error("pi rpc error: {0}")]
    PiRpc(String),
    #[error("process error: {0}")]
    Process(String),
    #[error("service unavailable: {0}")]
    ServiceUnavailable(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("timeout waiting for pi rpc response")]
    Timeout,
}

impl Error {
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::BadRequest(message.into())
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound(message.into())
    }

    pub fn session_busy(message: impl Into<String>) -> Self {
        Self::SessionBusy(message.into())
    }

    pub fn pi_rpc(message: impl Into<String>) -> Self {
        Self::PiRpc(message.into())
    }

    pub fn service_unavailable(message: impl Into<String>) -> Self {
        Self::ServiceUnavailable(message.into())
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, name) = match &self {
            Self::BadRequest(_) | Self::Json(_) => (StatusCode::BAD_REQUEST, "BadRequest"),
            Self::NotFound(_) => (StatusCode::NOT_FOUND, "NotFoundError"),
            Self::SessionBusy(_) => (StatusCode::CONFLICT, "SessionBusyError"),
            Self::ServiceUnavailable(_) => {
                (StatusCode::SERVICE_UNAVAILABLE, "ServiceUnavailableError")
            }
            Self::PiRpc(_) | Self::Process(_) | Self::Io(_) | Self::Sqlite(_) | Self::Timeout => {
                (StatusCode::INTERNAL_SERVER_ERROR, "InternalServerError")
            }
        };

        (
            status,
            Json(json!({
                "name": name,
                "data": {
                    "message": self.to_string(),
                }
            })),
        )
            .into_response()
    }
}
