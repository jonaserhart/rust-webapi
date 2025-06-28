use std::hash::{Hash, Hasher};
use serde::{Deserialize, Serialize};
use diesel::prelude::*;

#[derive(Debug, Serialize, Queryable, Selectable, Clone)]
#[diesel(table_name = crate::model::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub(crate) id: i32,
    pub(crate) username: String,
    pub(crate) email: String,
    #[serde(skip_serializing)]
    pub(crate) password: String
}

impl Hash for User {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

#[derive(Debug, Deserialize, Insertable)]
#[diesel(table_name = crate::model::schema::users)]
pub struct CreateUser {
    pub(crate) username: String,
    pub(crate) email: String,
    pub(crate) password: String
}
