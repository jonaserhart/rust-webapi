mod model;
mod repos;
mod routes;
mod common;

use std::sync::Arc;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use axum::Router;
use axum::routing::{get, post};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use crate::repos::user_repo::{CustomUserRepo, DynUserRepo};
use crate::routes::user_roues::{users_create, users_show};

use diesel_async::{
    pooled_connection::AsyncDieselConnectionManager,
};
use crate::common::encryption::{DynEncryptionService, EncryptionService};

#[derive(Clone)]
pub struct AppState {
    pub user_repo: DynUserRepo,
    pub encryption_service: DynEncryptionService,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "webapi=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Config
    let db_url = std::env::var("DATABASE_URL").unwrap();
    let db_salt = SaltString::generate(&mut OsRng);

    let config = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(db_url);
    let pool = bb8::Pool::builder().build(config).await.unwrap();

    // DI
    let user_repo = Arc::new(CustomUserRepo::new(pool)) as DynUserRepo;
    let encryption_service = Arc::new(EncryptionService::new(db_salt)) as DynEncryptionService;

    let state = AppState {
        user_repo,
        encryption_service,
    };

    let user_router = Router::new()
        .route("/", post(users_create))
        .route("/:id", get(users_show));

    let app = Router::new()
        .nest("/users", user_router)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    tracing::debug!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}
