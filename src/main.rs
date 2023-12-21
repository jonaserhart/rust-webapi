mod model;
mod repos;
mod routes;
mod common;

use std::sync::Arc;
use axum::Router;
use axum::routing::{get, post};
use diesel_async::RunQueryDsl;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use crate::repos::user_repo::{CustomUserRepo, DynUserRepo};
use crate::routes::user_routes::{users_create, users_show, user_authorize};
use dotenv::dotenv;

use diesel_async::{
    pooled_connection::AsyncDieselConnectionManager,
};
use crate::common::jwt::Keys;

#[derive(Clone)]
struct AppConfig {
    jwt_keys: Keys
}

impl AppConfig{
    pub fn new(jwt_keys: Keys) -> Self{
        Self{
            jwt_keys
        }
    }
}

#[derive(Clone)]
struct AppState {
    pub user_repo: DynUserRepo,
    pub app_config: AppConfig,
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

    if dotenv().is_err() {
        tracing::warn!("Could not read .env, using manually set env variables...");
    } else {
        tracing::debug!("Read .env")
    }

    // Config
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not provided!");

    let config = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(db_url);
    let pool = bb8::Pool::builder().build(config).await.unwrap();

    // DI
    let user_repo = Arc::new(CustomUserRepo::new(pool)) as DynUserRepo;

    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let keys = Keys::new(secret.as_bytes());

    let app_config = AppConfig{ jwt_keys: keys };

    let state = AppState {
        user_repo,
        app_config
    };

    let user_router = Router::new()
        .route("/", post(users_create))
        .route("/:id", get(users_show))
        .route("/authorize", post(user_authorize));

    let app = Router::new()
        .nest("/users", user_router)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    tracing::debug!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}
