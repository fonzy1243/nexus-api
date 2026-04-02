use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{
        SaltString,
        rand_core::{OsRng, RngCore},
    },
};
use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{AppError, Result};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid, // user id
    pub exp: i64,  // expiry (unix timestamp)
    pub version: i32,
}

pub fn create_token(user_id: Uuid, secret: &str, version: i32) -> Result<String> {
    let exp = (Utc::now() + Duration::minutes(10)).timestamp();
    let claims = Claims {
        sub: user_id,
        exp,
        version,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|_| AppError::BadRequest("Failed to create token".into()))
}

// Generate refresh token
pub fn create_refresh_token() -> Result<(String, String, String)> {
    let mut bytes = [0u8; 64];
    OsRng.fill_bytes(&mut bytes);
    let raw = BASE64_URL_SAFE_NO_PAD.encode(bytes);

    let mut id_bytes = [0u8; 16];
    OsRng.fill_bytes(&mut id_bytes);
    let token_id = BASE64_URL_SAFE_NO_PAD.encode(id_bytes);

    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(raw.as_bytes(), &salt)
        .map_err(|_| AppError::BadRequest("Failed to hash refresh token".into()))?
        .to_string();

    Ok((token_id, raw, hash))
}

pub fn verify_token(token: &str, secret: &str) -> Result<Claims> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|d| d.claims)
    .map_err(|_| AppError::Unauthorized)
}

pub fn verify_refresh_token(raw: &str, hash: &str) -> Result<()> {
    let parsed = PasswordHash::new(hash).map_err(|_| AppError::Unauthorized)?;
    Argon2::default()
        .verify_password(raw.as_bytes(), &parsed)
        .map_err(|_| AppError::Unauthorized)
}
