use std::fmt::Display;

use crate::model::auth::AuthResponse;
use crate::model::errors::{AppError, AuthError};
use crate::model::users::User;
use crate::AppState;
use async_trait::async_trait;
use axum::extract::{FromRequestParts, Request, State, FromRequest};
use axum::http::header::{COOKIE, SET_COOKIE};
use axum::http::request::Parts;
use axum::http::{HeaderMap, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::{Json, RequestPartsExt};
use axum_extra::headers::authorization::Bearer;
use axum_extra::headers::Authorization;
use axum_extra::TypedHeader;
use cookie::Cookie;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

const ONE_WEEK_IN_SECS: i64 = 604600;
const FIVE_MIN_IN_SECS: i64 = 300;

#[derive(Debug, Serialize, Deserialize)]
pub struct Role {}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenClaims {
    pub(crate) uid: i32,
    pub(crate) exp: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AccessTokenClaims {
    pub(crate) uid: i32,
    pub(crate) exp: i64,
    pub(crate) roles: Vec<Role>,
}

impl Display for RefreshTokenClaims {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "User: {}\nExp: {}", self.uid, self.exp)
    }
}

impl Display for AccessTokenClaims {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "User: {}\nExp: {}", self.uid, self.exp)
    }
}

async fn extract_token_data<T>(parts: &mut Parts, state: &AppState) -> Result<T, AppError>
where
    T: for<'de> Deserialize<'de>,
{
    let keys = &state.app_config.jwt_keys;
    // Extract the token from the authorization header
    let TypedHeader(Authorization(bearer)) = parts
        .extract::<TypedHeader<Authorization<Bearer>>>()
        .await
        .map_err(|_| AuthError::TokenMissing)?;
    // Decode the user data
    let token_data = decode::<T>(bearer.token(), &keys.decoding, &Validation::default())
        .map_err(|_| AuthError::InvalidToken)?;

    Ok(token_data.claims)
}

#[async_trait]
impl FromRequestParts<AppState> for AccessTokenClaims {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        extract_token_data::<AccessTokenClaims>(parts, state).await
    }
}

#[async_trait]
impl FromRequestParts<AppState> for RefreshTokenClaims {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let cookie_header = parts
            .headers
            .get(COOKIE)
            .ok_or(AuthError::MissingCookie)?
            .to_str()
            .map_err(|_| AuthError::InvalidCookie)?;

        // Parse the cookie
        let cookies = cookie_header
            .split(';')
            .filter_map(|s| Cookie::parse(s).ok())
            .collect::<Vec<_>>();

        // Find the specific cookie for the refresh token
        let refresh_cookie = cookies
            .iter()
            .find(|c| c.name() == "refresh_token")
            .ok_or(AuthError::MissingCookie)?;

        // Decode the JWT token
        let token_data = decode::<RefreshTokenClaims>(
            refresh_cookie.value(),
            &state.app_config.jwt_keys.decoding,
            &Validation::default(),
        )
        .map_err(|_| AuthError::InvalidToken)?;

        Ok(token_data.claims)
    }
}

#[derive(Clone)]
pub struct Keys {
    pub encoding: EncodingKey,
    pub decoding: DecodingKey,
}

impl Keys {
    pub(crate) fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

pub fn generate_token(user: User, jwt_keys: &Keys) -> Result<impl IntoResponse, AppError> {
    let now = chrono::offset::Utc::now().timestamp();

    let claims = AccessTokenClaims {
        exp: now + FIVE_MIN_IN_SECS,
        uid: user.id,
        roles: vec![],
    };

    let refresh_claims = RefreshTokenClaims {
        exp: now + ONE_WEEK_IN_SECS,
        uid: user.id,
    };

    let jwt = encode(&Header::default(), &claims, &jwt_keys.encoding)
        .map_err(|_| AuthError::TokenCreation)?;

    let body = AuthResponse {
        user_id: user.id,
        token: jwt,
    };

    let refresh_token = encode(&Header::default(), &refresh_claims, &jwt_keys.encoding)
        .map_err(|_| AuthError::TokenCreation)?;

    let cookie = Cookie::build(("refresh_token", refresh_token.clone()))
        .secure(true)
        .http_only(true)
        .build();

    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, cookie.to_string().parse().unwrap());

    Ok((headers, Json(body)))
}

async fn auth_middleware(State(state): State<AppState>, mut req: Request, next: Next) -> Result<Response, StatusCode> {
    
    let parts = RequestParts::new() ;

    let TypedHeader(Authorization(bearer)) = TypedHeader::<Authorization<Bearer>>::from_request(req, &state).await
        .map_err(|_| {StatusCode::UNAUTHORIZED})?;

    let token = bearer.token();
    
    let token_data = decode::<AccessTokenClaims>(token, &state.app_config.jwt_keys.decoding, &Validation::default())
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let current_user = state.user_repo.find(token_data.claims.uid).await.map_err(|_| {StatusCode::UNAUTHORIZED})?;

    req.extensions_mut().insert(current_user);
    Ok(next.run(req).await)
}
