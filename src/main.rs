mod handler;
mod portfolio;
mod types;

use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;

use axum::Router;
use axum::routing::{get, post};
use tower_http::services::{ServeDir, ServeFile};

use crate::handler::{
    create_executions, create_portfolio, get_history, get_portfolio_calc, get_positions, list_marketdata, list_portfolios,
};
use crate::portfolio::seed::load_appstate;

fn static_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui/dist")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let ctx = Arc::new(load_appstate());

    // Static files to serve ui
    let static_path = static_dir();
    let serve_dir = ServeDir::new(static_path.clone()).fallback(ServeFile::new(static_path.join("index.html")));

    // REST routes
    let app = Router::new()
        .route("/portfolios", get(list_portfolios).post(create_portfolio))
        .route("/portfolios/{id}/positions", get(get_positions))
        .route("/portfolios/{id}/history", get(get_history))
        .route("/portfolios/{id}/executions", post(create_executions))
        .route("/portfolios/{id}/calc", get(get_portfolio_calc))
        .route("/marketdata", get(list_marketdata))
        .with_state(ctx)
        .fallback_service(serve_dir);

    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("Listening on {addr}");
    axum::serve(listener, app).await?;
    Ok(())
}
