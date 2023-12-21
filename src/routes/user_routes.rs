use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::http::header::SET_COOKIE;
use axum::Json;
use axum::response::IntoResponse;
use jsonwebtoken::{encode, Header};
use crate::AppState;
use crate::model::auth::{AuthResponse, AuthSubmission};
use crate::model::errors::{AppError, AuthError};
use crate::model::users::{CreateUser, User};
use crate::common::encryption;
use crate::common::encryption::validate;
use crate::common::jwt::Claims;
use cookie::Cookie;

const ONE_WEEK_IN_SECS: i64 = 604600;
const FIVE_MIN_IN_SECS: i64 = 300;

pub async fn users_show(
    Path(user_id): Path<i32>,
    State(state): State<AppState>
) -> Result<Json<User>, AppError> {
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


pub async fn user_authorize(
    State(state): State<AppState>,
    Json(auth_info): Json<AuthSubmission>
) -> Result<impl IntoResponse, AppError> {
    if auth_info.user_or_email.is_empty() {
        return Err(AuthError::MissingUserName.into());
    }
    if auth_info.password.is_empty() {
        return Err(AuthError::MissingUserName.into());
    }

    let user = state.user_repo.find_by_username_or_email(&auth_info.user_or_email)
        .await
        .map_err(|_| { AuthError::UserNotFound })?;

    validate(&auth_info.password, &user.password)
        .map_err(|_| { AuthError::IncorrectPassword })?;

    let now = chrono::offset::Utc::now().timestamp();

    let claims = Claims {
        exp: now + FIVE_MIN_IN_SECS,
        uid: user.id
    };

    let refresh_claims = Claims{
        exp: now + ONE_WEEK_IN_SECS,
        uid: user.id
    };

    let jwt = encode(&Header::default(), &claims, &state.app_config.jwt_keys.encoding)
        .map_err(|_| AuthError::TokenCreation)?;

    let body = AuthResponse{
        user_id: user.id,
        token: jwt,
    };

    let refresh_token = encode(&Header::default(), &refresh_claims, &state.app_config.jwt_keys.encoding)
        .map_err(|_| AuthError::TokenCreation)?;

    let cookie = Cookie::build(("refresh_token", refresh_token.clone()))
        .secure(true)
        .http_only(true)
        .build();

    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, cookie.to_string().parse().unwrap());

    Ok((headers, Json(body)))
}
