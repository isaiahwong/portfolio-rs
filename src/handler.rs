use crate::portfolio::position::PositionSnapshot;
use crate::portfolio::types::InstrumentId;
use crate::portfolio::{
    Portfolio,
    types::{Currency, Execution},
};
use crate::types::{AddExecReq, AppError, CreatePortfolioReq, CreatePortfolioRes, PnlQuery, PnlResponse, PortfolioPath};
use axum::{
    Json,
    extract::{Path, Query, State},
};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub async fn create_portfolio(
    State(state): State<Arc<crate::portfolio::seed::AppState>>,
    Json(req): Json<CreatePortfolioReq>,
) -> Result<Json<CreatePortfolioRes>, AppError> {
    let name = req.name.unwrap_or_else(|| Uuid::now_v7().into());
    let portfolio = Portfolio::new(state.providers.clone());
    state
        .portfolios
        .write()
        .unwrap()
        .insert(name.clone(), Arc::new(Mutex::new(portfolio)));

    Ok(Json(CreatePortfolioRes { name }))
}

pub async fn get_positions(
    State(state): State<Arc<crate::portfolio::seed::AppState>>,
    Path(path): Path<PortfolioPath>,
) -> Result<Json<Vec<PositionSnapshot>>, AppError> {
    let portfolios = state.portfolios.read().unwrap();
    let portfolio = portfolios
        .get(&path.id)
        .cloned()
        .ok_or_else(|| AppError::NotFound("Portfolio not found".into()))?;

    let positions = portfolio.lock().unwrap().positions();
    Ok(Json(positions))
}

pub async fn get_history(
    State(state): State<Arc<crate::portfolio::seed::AppState>>,
    Path(path): Path<PortfolioPath>,
) -> Result<Json<Vec<PositionSnapshot>>, AppError> {
    let portfolios = state.portfolios.read().unwrap();
    let portfolio = portfolios
        .get(&path.id)
        .cloned()
        .ok_or_else(|| AppError::NotFound("Portfolio not found".into()))?;

    let history = portfolio.lock().unwrap().history();
    Ok(Json(history))
}

pub async fn add_executions(
    State(state): State<Arc<crate::portfolio::seed::AppState>>,
    Path(path): Path<PortfolioPath>,
    Json(req): Json<AddExecReq>,
) -> Result<Json<()>, AppError> {
    let portfolio_id = path.id.clone();
    let portfolio = {
        let portfolios = state.portfolios.read().unwrap();
        portfolios
            .get(&portfolio_id)
            .cloned()
            .ok_or_else(|| AppError::NotFound("Portfolio not found".into()))?
    };

    let execution = Execution {
        id: Uuid::now_v7().to_string(),
        instrument_id: req.instrument_id,
        portfolio_id: portfolio_id.clone(),
        side: req.side,
        qty: req.qty,
        px: req.px,
    };

    portfolio.lock().unwrap().on_exec(execution);

    Ok(Json(()))
}

pub async fn get_unrealized_pnl(
    State(state): State<Arc<crate::portfolio::seed::AppState>>,
    Path(path): Path<PortfolioPath>,
    Query(query): Query<PnlQuery>,
) -> Result<Json<PnlResponse>, AppError> {
    let portfolio_id = path.id;
    let currency_code = query.currency.as_deref().unwrap_or("USD");
    let target = Currency {
        code: currency_code.into(),
    };

    let portfolio = {
        let portfolios = state.portfolios.read().unwrap();
        portfolios
            .get(&portfolio_id)
            .cloned()
            .ok_or_else(|| AppError::NotFound("Portfolio not found".into()))?
    };

    let pnl = portfolio
        .lock()
        .unwrap()
        .calc_total_unrealized_pnl(target.clone())
        .map_err(|e| AppError::Internal(format!("{:?}", e)))?;

    Ok(Json(PnlResponse {
        currency: target.code,
        unrealized_pnl: pnl,
    }))
}

pub async fn list_currencies(State(state): State<Arc<crate::portfolio::seed::AppState>>) -> Json<Vec<Currency>> {
    Json(state.providers.currencies.clone())
}

pub async fn list_instruments(State(state): State<Arc<crate::portfolio::seed::AppState>>) -> Json<Vec<InstrumentId>> {
    Json(state.providers.instrument_ids())
}

pub async fn list_portfolios(State(state): State<Arc<crate::portfolio::seed::AppState>>) -> Json<Vec<String>> {
    let portfolios = state.portfolios.read().unwrap();
    Json(portfolios.keys().cloned().collect())
}
