use crate::portfolio::types::Currency;
use crate::portfolio::types::InstrumentId;
use crate::portfolio::types::Side;
use serde::{Deserialize, Serialize};

/// Request for creating a new portfolio.
#[derive(Deserialize)]
pub struct CreatePortfolioRequest {
    pub name: Option<String>,
}

/// Response for a created portfolio.
#[derive(Serialize)]
pub struct CreatePortfolioResponse {
    pub name: String,
}

/// Request for adding an execution to a portfolio.
#[derive(Deserialize)]
pub struct AddExecRequest {
    pub instrument_id: InstrumentId,
    pub side: Side,
    pub qty: f64,
    pub px: f64,
}

/// Result of portfolio calculation.
#[derive(Serialize)]
pub struct CalcResponse {
    pub currency: String,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
    pub total_value: f64,
}

/// Query params for portfolio calculation.
#[derive(Deserialize)]
pub struct CalcQuery {
    pub currency: Option<String>,
}

/// Response for current market data snapshots.
#[derive(Serialize)]
pub struct MarketDataResponse {
    pub currencies: Vec<Currency>,
    pub instruments: Vec<InstrumentId>,
    pub mid_prices: Vec<(InstrumentId, f64)>,
}
