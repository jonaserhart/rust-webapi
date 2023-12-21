use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct AuthSubmission {
    pub user_or_email: String,
    pub password: String
}

#[derive(Debug, Serialize)]
pub struct TokenAndUserId {
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub(crate) token: String,
    pub(crate) user_id: i32,
}
