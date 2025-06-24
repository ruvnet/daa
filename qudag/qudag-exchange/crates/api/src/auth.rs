//! Authentication middleware and utilities

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{header::AUTHORIZATION, request::Parts, StatusCode},
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use crate::error::ApiError;

/// JWT claims structure
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // Subject (account ID)
    pub exp: usize,  // Expiry time
    pub iat: usize,  // Issued at
}

/// Auth token extractor
pub struct AuthToken(pub String);

#[async_trait]
impl<S> FromRequestParts<S> for AuthToken
where
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or(ApiError::Unauthorized)?;

        if !auth_header.starts_with("Bearer ") {
            return Err(ApiError::Unauthorized);
        }

        let token = auth_header.trim_start_matches("Bearer ");
        Ok(AuthToken(token.to_string()))
    }
}

/// Generate a JWT token for an account
pub fn generate_token(account_id: &str, secret: &[u8]) -> Result<String, ApiError> {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: account_id.to_string(),
        exp: expiration,
        iat: chrono::Utc::now().timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret),
    )
    .map_err(|_| ApiError::Internal(anyhow::anyhow!("Failed to generate token")))
}

/// Validate a JWT token
pub fn validate_token(token: &str, secret: &[u8]) -> Result<Claims, ApiError> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret),
        &Validation::default(),
    )
    .map(|data| data.claims)
    .map_err(|_| ApiError::Unauthorized)
}