use crate::portfolio::position::PositionSnapshot;
use crate::portfolio::seed::AppState;
use crate::portfolio::{
    Portfolio,
    types::{Currency, Execution},
};
use crate::types::{AddExecRequest, CalcQuery, CalcResponse, CreatePortfolioRequest, CreatePortfolioResponse, MarketDataResponse};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub type Error = (StatusCode, String);

/// Creates a new portfolio with an optional name.
pub async fn create_portfolio(
    State(ctx): State<Arc<AppState>>,
    Json(req): Json<CreatePortfolioRequest>,
) -> Result<Json<CreatePortfolioResponse>, Error> {
    let name = req.name.unwrap_or_else(|| Uuid::now_v7().into());
    let portfolio = Portfolio::new(ctx.marketdata.clone());
    ctx.portfolios
        .write()
        .unwrap()
        .insert(name.clone(), Arc::new(Mutex::new(portfolio)));

    Ok(Json(CreatePortfolioResponse { name }))
}

/// Returns a list of all portfolio names.
pub async fn list_portfolios(State(ctx): State<Arc<AppState>>) -> Json<Vec<String>> {
    let portfolios = ctx.portfolios.read().unwrap();
    let mut keys: Vec<_> = portfolios.keys().cloned().collect();
    keys.sort_by_key(|a| a.to_lowercase());
    Json(keys)
}

/// Returns the current open positions for a specific portfolio.
pub async fn get_positions(State(ctx): State<Arc<AppState>>, Path(id): Path<String>) -> Result<Json<Vec<PositionSnapshot>>, Error> {
    let portfolio = get_portfolio(&ctx, &id)?;
    let positions = portfolio.lock().unwrap().positions();
    Ok(Json(positions))
}

/// Returns the historical snapshots of closed positions for a specific portfolio.
pub async fn get_history(State(ctx): State<Arc<AppState>>, Path(id): Path<String>) -> Result<Json<Vec<PositionSnapshot>>, Error> {
    let portfolio = get_portfolio(&ctx, &id)?;
    let history = portfolio.lock().unwrap().history();
    Ok(Json(history))
}

/// Adds a trade execution record to a specific portfolio.
pub async fn create_executions(
    State(ctx): State<Arc<AppState>>,
    Path(portfolio_id): Path<String>,
    Json(req): Json<AddExecRequest>,
) -> Result<Json<()>, Error> {
    let portfolio = get_portfolio(&ctx, &portfolio_id)?;
    let execution = Execution::new(req.instrument_id, portfolio_id.clone(), req.px, req.qty, req.side);

    portfolio.lock().unwrap().on_exec(execution).map_err(to_error)?;

    Ok(Json(()))
}

/// Calculates and returns portfolio metrics (PnL, total value) in a target currency.
pub async fn get_portfolio_calc(
    State(ctx): State<Arc<AppState>>,
    Path(portfolio_id): Path<String>,
    Query(query): Query<CalcQuery>,
) -> Result<Json<CalcResponse>, Error> {
    let portfolio = get_portfolio(&ctx, &portfolio_id)?;
    let portfolio = portfolio.lock().unwrap();

    let target = Currency::new(query.currency.as_deref().unwrap_or("USD"));

    Ok(Json(CalcResponse {
        currency: target.code.clone(),
        unrealized_pnl: portfolio.calc_total_unrealized_pnl(target.clone()).map_err(to_error)?,
        realized_pnl: portfolio.calc_total_realized_pnl(target.clone()).map_err(to_error)?,
        total_value: portfolio.calc_total_value(target).map_err(to_error)?,
    }))
}

/// Returns current market data including currencies, instruments, and mid prices.
pub async fn list_marketdata(State(ctx): State<Arc<AppState>>) -> Json<MarketDataResponse> {
    Json(MarketDataResponse {
        currencies: ctx.marketdata.currencies.clone(),
        instruments: ctx.marketdata.instrument_ids(),
        mid_prices: ctx.marketdata.mid_prices(),
    })
}

/// Helper fn to retrieve a portfolio.
fn get_portfolio(ctx: &AppState, id: &str) -> Result<Arc<Mutex<Portfolio>>, Error> {
    ctx.portfolios
        .read()
        .unwrap()
        .get(id)
        .cloned()
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Portfolio not found".into()))
}

fn to_error<E: std::fmt::Debug>(e: E) -> Error {
    (StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}", e))
}
