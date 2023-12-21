use std::sync::Arc;
use async_trait::async_trait;
use bb8::Pool;

use diesel::{QueryDsl, SelectableHelper};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use crate::model::errors::UserRepoError;
use crate::model::users::{CreateUser, User};

#[async_trait]
pub trait UserRepo {
    async fn find(&self, user_id: i32) -> Result<User, UserRepoError>;
    async fn create(&self, _params: CreateUser) -> Result<User, UserRepoError>;
}

// implementations
pub struct CustomUserRepo {
    pool:  Pool<AsyncDieselConnectionManager<AsyncPgConnection>>
}

impl CustomUserRepo {
    pub fn new(pool:  Pool<AsyncDieselConnectionManager<AsyncPgConnection>>) -> Self {
        CustomUserRepo{pool}
    }
}

#[async_trait]
impl UserRepo for CustomUserRepo {
    async fn find(&self, q_user_id: i32) -> Result<User, UserRepoError> {
        use crate::model::schema::users::dsl::*;
        let mut conn = self.pool.get().await.expect("Could not get pool connection");

        let user = users
            .find(q_user_id)
            .select(User::as_select())
            .first(&mut conn)
            .await;

        return if user.is_ok() {
            Ok(user.unwrap())
        } else {
            Err(UserRepoError::NotFound)
        }
    }

    async fn create(&self, _params: CreateUser) -> Result<User, UserRepoError> {
        use crate::model::schema::users;
        let mut conn = self.pool.get().await.expect("Could not get pool connection");

        let saved = diesel::insert_into(users::table)
            .values(&_params)
            .returning(User::as_returning())
            .get_result(&mut conn)
            .await;

        match saved {
            Err(_) => Err(UserRepoError::InvalidUserName),
            Ok(u) => Ok(u)
        }
    }
}

pub type DynUserRepo = Arc<dyn UserRepo + Send + Sync>;
