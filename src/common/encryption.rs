use std::sync::Arc;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::SaltString;
use crate::model::errors::EncryptionError;

pub trait Encryption {
    fn hash(&self, str: &str) -> Result<String, EncryptionError>;
    fn validate(&self, str: &str, hash: &str) -> Result<(), EncryptionError>;
}

pub struct EncryptionService {
    salt: SaltString
}

impl EncryptionService {
    pub fn new(salt_string: SaltString) -> Self {
        EncryptionService{salt: salt_string}
    }
}

impl Encryption for EncryptionService {
    fn hash(&self, str: &str) -> Result<String, EncryptionError> {
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(str.as_ref(), &self.salt);

        match password_hash {
            Ok(hash) => Ok(hash.to_string()),
            Err(_) => Err(EncryptionError::HashError)
        }
    }

    fn validate(&self, str: &str, hash: &str) -> Result<(), EncryptionError> {
        let argon2 = Argon2::default();
        let parsed_hash = PasswordHash::new(&hash);

        if parsed_hash.is_err() {
            return Err(EncryptionError::VerifyError)
        }

        let result = argon2.verify_password(str.as_ref(), &parsed_hash.unwrap());

        if result.is_err() {
            return Err(EncryptionError::VerifyError);
        }

        Ok(())
    }
}

pub type DynEncryptionService = Arc<dyn Encryption + Send + Sync>;

#[cfg(test)]
mod tests {
    use super::*;
    use argon2::{password_hash::{rand_core::OsRng}};

    #[test]
    fn test_hash_successful() {
        let salt_string = SaltString::generate(&mut OsRng);
        let svc = EncryptionService{salt: salt_string};

        let string_to_hash = "string_to_hash";

        let hashed = svc.hash(string_to_hash).unwrap();

        svc.validate(string_to_hash, &hashed).expect("Hash could not be validated!")
    }

    #[test]
    fn test_hash_not_successful() {
        let salt_string = SaltString::generate(&mut OsRng);
        let svc = EncryptionService{salt: salt_string};

        let string_to_hash = "string_to_hash";

        let hashed = svc.hash(string_to_hash).unwrap();

        let expected = Err(EncryptionError::VerifyError);

        let wrong_string_to_verify = "wrong_string";

        let result = svc.validate(wrong_string_to_verify, &hashed);;

        assert_eq!(expected, result);
    }
}
