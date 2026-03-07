mod handler;
mod portfolio;
mod types;

use std::sync::Arc;

use axum::Router;
use axum::routing::{get, post};

use crate::handler::{
    add_executions, create_portfolio, get_history, get_positions, get_unrealized_pnl, list_currencies, list_instruments, list_portfolios,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Starting server on 0.0.0.0:3000");
    let state = Arc::new(crate::portfolio::seed::load());

    let app = Router::new()
        .route("/portfolios", get(list_portfolios).post(create_portfolio))
        .route("/portfolios/{id}/positions", get(get_positions))
        .route("/portfolios/{id}/history", get(get_history))
        .route("/portfolios/{id}/executions", post(add_executions))
        .route("/portfolios/{id}/pnl", get(get_unrealized_pnl))
        .route("/currencies", get(list_currencies))
        .route("/instruments", get(list_instruments))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("Listening on http://0.0.0.0:3000");
    axum::serve(listener, app).await?;
    Ok(())
}
