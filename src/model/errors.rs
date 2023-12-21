use axum::http::StatusCode;
use axum::Json;
use axum::response::{IntoResponse, Response};
use serde_json::json;

#[derive(Debug)]
pub enum UserRepoError {
    NotFound,
    InvalidUserName
}

#[derive(Debug, PartialEq)]
pub enum EncryptionError<> {
    HashError,
    VerifyError
}

pub enum AppError {
    UserRepoError(UserRepoError),
    EncryptionError(EncryptionError),
}

// implementations

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::UserRepoError(UserRepoError::NotFound) => {
                (StatusCode::NOT_FOUND, "User not found")
            }
            AppError::UserRepoError(UserRepoError::InvalidUserName) => {
                (StatusCode::UNPROCESSABLE_ENTITY, "Invalid username")
            }
            _ => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Unknown server error")
            }
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}

impl From<UserRepoError> for AppError {
    fn from(inner: UserRepoError) -> Self {
        AppError::UserRepoError(inner)
    }
}
impl From<EncryptionError> for AppError {
    fn from(inner: EncryptionError) -> Self {
        AppError::EncryptionError(inner)
    }
}
