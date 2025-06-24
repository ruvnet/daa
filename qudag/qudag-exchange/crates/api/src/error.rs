//! API error handling

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Account not found")]
    AccountNotFound,
    
    #[error("Insufficient balance")]
    InsufficientBalance,
    
    #[error("Invalid transaction")]
    InvalidTransaction(String),
    
    #[error("Unauthorized")]
    Unauthorized,
    
    #[error("Internal server error")]
    Internal(#[from] anyhow::Error),
}

#[derive(Debug, Serialize, Deserialize)]
struct ErrorResponse {
    error: String,
    message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error) = match self {
            ApiError::AccountNotFound => (StatusCode::NOT_FOUND, "NOT_FOUND"),
            ApiError::InsufficientBalance => (StatusCode::BAD_REQUEST, "INSUFFICIENT_BALANCE"),
            ApiError::InvalidTransaction(_) => (StatusCode::BAD_REQUEST, "INVALID_TRANSACTION"),
            ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED"),
            ApiError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR"),
        };
        
        let body = Json(ErrorResponse {
            error: error.to_string(),
            message: self.to_string(),
        });
        
        (status, body).into_response()
    }
}