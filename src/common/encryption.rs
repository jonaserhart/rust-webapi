use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use crate::model::errors::EncryptionError;

pub fn hash(str: &str) -> Result<String, EncryptionError> {
    let argon2 = Argon2::default();
    let salt_string = SaltString::generate(&mut OsRng);
    let password_hash = argon2.hash_password(str.as_ref(), &salt_string);

    match password_hash {
        Ok(hash) => Ok(hash.to_string()),
        Err(_) => Err(EncryptionError::HashError)
    }
}

pub fn validate(str: &str, hash: &str) -> Result<(), EncryptionError> {
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
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_successful() {
        let string_to_hash = "string_to_hash";

        let hashed = hash(string_to_hash).unwrap();

        validate(string_to_hash, &hashed).expect("Hash could not be validated!")
    }

    #[test]
    fn test_validate_not_successful() {
        let string_to_hash = "string_to_hash";

        let hashed = hash(string_to_hash).unwrap();

        let expected = Err(EncryptionError::VerifyError);

        let wrong_string_to_verify = "wrong_string";

        let result = validate(wrong_string_to_verify, &hashed);

        assert_eq!(expected, result);
    }
}
