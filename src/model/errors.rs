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
pub enum EncryptionError {
    HashError,
    VerifyError
}

#[derive(Debug)]
pub enum AuthError {
    MissingUserName,
    MissingPassword,
    UserNotFound,
    IncorrectPassword,
    TokenCreation,
    InvalidToken,
    TokenMissing,
    MissingCookie,
    InvalidCookie,
}

pub enum AppError {
    UserRepoError(UserRepoError),
    EncryptionError(EncryptionError),
    AuthError(AuthError),
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
            AppError::AuthError(AuthError::IncorrectPassword) => {
                (StatusCode::UNAUTHORIZED, "Invalid password")
            }
            AppError::AuthError(AuthError::UserNotFound) => {
                (StatusCode::UNAUTHORIZED, "User not found")
            }
            AppError::AuthError(AuthError::MissingUserName) => {
                (StatusCode::UNAUTHORIZED, "Username is missing")
            }
            AppError::AuthError(AuthError::MissingPassword) => {
                (StatusCode::UNAUTHORIZED, "Password is missing")
            }
            AppError::AuthError(AuthError::InvalidToken) => {
                (StatusCode::UNAUTHORIZED, "Invalid token")
            }
            AppError::AuthError(AuthError::TokenMissing) => {
                (StatusCode::UNAUTHORIZED, "Missing token")
            }
            AppError::AuthError(AuthError::MissingCookie) => {
                (StatusCode::UNAUTHORIZED, "Missing cookie")
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

impl From<AuthError> for AppError {
    fn from(inner: AuthError) -> Self {
        AppError::AuthError(inner)
    }
}
