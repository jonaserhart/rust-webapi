use std::sync::Arc;
use async_trait::async_trait;
use bb8::Pool;

use diesel::SelectableHelper;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use uuid::Uuid;
use crate::model::errors::UserRepoError;
use crate::model::users::{CreateUser, User};

#[async_trait]
pub trait UserRepo {
    async fn find(&self, user_id: Uuid) -> Result<User, UserRepoError>;
    async fn create(&self, _params: CreateUser) -> Result<User, UserRepoError>;
}

// implementations
pub struct CustomUserRepo {
    pub(crate) pool:  Pool<AsyncDieselConnectionManager<AsyncPgConnection>>
}

#[async_trait]
impl UserRepo for CustomUserRepo {
    async fn find(&self, user_id: Uuid) -> Result<User, UserRepoError> {
        unimplemented!()
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
            Err(e) => Err(UserRepoError::InvalidUserName),
            Ok(u) => Ok(u)
        }
    }
}

pub type DynUserRepo = Arc<dyn UserRepo + Send + Sync>;
