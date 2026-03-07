use crate::portfolio::types::InstrumentId;
use crate::portfolio::types::Side;
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};

pub enum AppError {
    NotFound(String),
    Internal(String),
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(ErrorResponse { error: error_message });

        (status, body).into_response()
    }
}

#[derive(Deserialize)]
pub struct CreatePortfolioReq {
    pub name: Option<String>,
}

#[derive(Serialize)]
pub struct CreatePortfolioRes {
    pub name: String,
}

#[derive(Deserialize)]
pub struct AddExecReq {
    pub instrument_id: InstrumentId,
    pub side: Side,
    pub qty: f64,
    pub px: f64,
}

#[derive(Serialize)]
pub struct PnlResponse {
    pub currency: String,
    pub unrealized_pnl: f64,
}

#[derive(Deserialize)]
pub struct PortfolioPath {
    pub id: String,
}

#[derive(Deserialize)]
pub struct PnlQuery {
    pub currency: Option<String>,
}
