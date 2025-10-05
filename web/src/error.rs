use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use tracing::error;

#[derive(Debug)]
pub enum ApiError {
    Unauthorized,
    Forbidden,
    BadRequest(String),
    Internal(anyhow::Error),
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        match self {
            ApiError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                Json(ErrorBody {
                    error: "unauthorized".to_string(),
                }),
            )
                .into_response(),
            ApiError::Forbidden => (
                StatusCode::FORBIDDEN,
                Json(ErrorBody {
                    error: "forbidden".to_string(),
                }),
            )
                .into_response(),
            ApiError::BadRequest(message) => {
                (StatusCode::BAD_REQUEST, Json(ErrorBody { error: message })).into_response()
            }
            ApiError::Internal(err) => {
                error!(?err, "internal_api_error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorBody {
                        error: "internal error".to_string(),
                    }),
                )
                    .into_response()
            }
        }
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(err: anyhow::Error) -> Self {
        let message = err.to_string();
        if message.contains("amount exceeds") || message.contains("daily cap") {
            ApiError::BadRequest(message)
        } else {
            ApiError::Internal(err)
        }
    }
}
