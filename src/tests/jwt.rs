use crate::error::{ApiError, Error};
use crate::jwt::{Claims, decode_jwt};
use crate::message::Message;
use std::env;

#[test]
#[serial_test::serial]
fn decode_jwt_valid_token() {
    let secret = "mysecret";
    let claims = Claims {
        prompt: "test prompt".to_string(),
        exp: 9999999999,
    };
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap();
    let decoded = decode_jwt(&token, secret).unwrap();
    assert_eq!(decoded.prompt, "test prompt");
}

#[test]
fn decode_jwt_invalid_token() {
    let secret = "mysecret";
    let token = "invalid.token.value";
    let result = decode_jwt(token, secret);
    assert!(matches!(result, Err(Error::InvalidJWTCredentials(_))));
}

#[test]
#[serial_test::serial]
fn message_encrypted_variant_decrypts() {
    let secret = "testsecret";
    unsafe { env::set_var("JWT_SECRET", secret) };
    let claims = Claims {
        prompt: "encrypted prompt".to_string(),
        exp: 9999999999,
    };
    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap();
    let msg = Message::Encrypted(token);
    let result: String = msg.try_into().unwrap();
    assert_eq!(result, "encrypted prompt");
    unsafe { env::remove_var("JWT_SECRET") };
}

#[test]
#[serial_test::serial]
fn message_encrypted_variant_no_secret() {
    unsafe { env::remove_var("JWT_SECRET") };
    let msg = Message::Encrypted("sometoken".to_string());
    let result: Result<String, ApiError> = msg.try_into();
    assert_eq!(
        result.unwrap_err().to_string(),
        ApiError::from(Error::NoJWTSecretFound).to_string()
    ); // NoJWTSecretFound maps to 500
}
