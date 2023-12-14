mod model;
mod repos;
mod routes;

use std::sync::Arc;
use axum::Router;
use axum::routing::{get, post};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use crate::repos::user_repo::{CustomUserRepo, DynUserRepo};
use crate::routes::user_roues::{users_create, users_show};

use diesel::prelude::*;
use diesel_async::{
    pooled_connection::AsyncDieselConnectionManager, RunQueryDsl,
};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "webapi=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db_url = std::env::var("DATABASE_URL").unwrap();

    let config = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(db_url);
    let pool = bb8::Pool::builder().build(config).await.unwrap();

    let user_repo = Arc::new(CustomUserRepo{pool}) as DynUserRepo;

    let app = Router::new()
        .route("/users/:id", get(users_show))
        .route("/users", post(users_create))
        .with_state(user_repo);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    tracing::debug!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}
