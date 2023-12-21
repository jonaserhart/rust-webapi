use axum::extract::{Path, State};
use axum::Json;
use crate::AppState;
use crate::model::errors::{AppError};
use crate::model::users::{CreateUser, User};

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
    let pw_hash = state.encryption_service.hash(&params.password)?;

    let user = state.user_repo.create(CreateUser{password: pw_hash, ..params}).await?;

    Ok(user.into())
}
