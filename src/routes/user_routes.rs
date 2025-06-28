use axum::extract::{Path, State};
use axum::Json;
use axum::response::IntoResponse;
use crate::AppState;
use crate::model::auth::AuthSubmission;
use crate::model::errors::{AppError, AuthError};
use crate::model::users::{CreateUser, User};
use crate::common::encryption;
use crate::common::encryption::validate;
use crate::common::jwt::{RefreshTokenClaims, AccessTokenClaims, generate_token};



pub async fn get_user_by_id(
    Path(user_id): Path<i32>,
    claims: AccessTokenClaims,
    State(state): State<AppState>
) -> Result<Json<User>, AppError> {
    tracing::info!("Claims: {claims}");
    let user = state.user_repo.find(user_id).await?;

    Ok(user.into())
}

pub async fn users_create(
    State(state): State<AppState>,
    Json(params): Json<CreateUser>
) -> Result<Json<User>, AppError> {
    let pw_hash = encryption::hash(&params.password)?;

    let user = state.user_repo.create(CreateUser{password: pw_hash, ..params}).await?;

    Ok(user.into())
}

pub async fn authorize(
    State(state): State<AppState>,
    Json(auth_info): Json<AuthSubmission>
) -> Result<impl IntoResponse, AppError> {
    if auth_info.user_or_email.is_empty() {
        return Err(AuthError::MissingUserName.into());
    }
    if auth_info.password.is_empty() {
        return Err(AuthError::MissingPassword.into());
    }

    let user = state.user_repo.find_by_username_or_email(&auth_info.user_or_email)
        .await
        .map_err(|_| { AuthError::UserNotFound })?;

    validate(&auth_info.password, &user.password)
        .map_err(|_| { AuthError::IncorrectPassword })?;

    let response = generate_token(user, &state.app_config.jwt_keys)?;

    Ok(response)
}

pub async fn refresh_token(
    refresh_token_claims: RefreshTokenClaims,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let user = state.user_repo.find(refresh_token_claims.uid)
        .await
        .map_err(|_| { AuthError::UserNotFound })?;

    let result = generate_token(user, &state.app_config.jwt_keys)?;

    Ok(result)
}

