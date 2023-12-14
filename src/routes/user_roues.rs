use axum::extract::{Path, State};
use axum::Json;
use uuid::Uuid;
use crate::model::errors::AppError;
use crate::model::users::{CreateUser, User};
use crate::repos::user_repo::DynUserRepo;

pub async fn users_show(
    Path(user_id): Path<Uuid>,
    State(user_repo): State<DynUserRepo>
) -> Result<Json<User>, AppError> {
    let user = user_repo.find(user_id).await?;

    Ok(user.into())
}

pub async fn users_create(
    State(user_repo): State<DynUserRepo>,
    Json(params): Json<CreateUser>
) -> Result<Json<User>, AppError> {
    let user = user_repo.create(params).await?;

    Ok(user.into())
}
