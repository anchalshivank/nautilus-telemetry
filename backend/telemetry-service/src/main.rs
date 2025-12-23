use axum::{Router, routing::get};
use sqlx::PgPool;
use std::net::SocketAddr;
use telemetry_service::tracing::init_logging;
use telemetry_service::{
    database::get_pool,
    routes::{api_routes, root},
    state::AppState,
};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logging()?;

    dotenv::dotenv().ok();

    info!("Starting Telemetry Service");

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool: PgPool = get_pool(database_url, 5).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;
    info!("Migrations done");

    let state = AppState::builder().db(pool.clone()).build();

    let app = Router::new()
        .route("/", get(root))
        .nest("/api/v1", api_routes(state.clone()))
        .with_state(state);

    let port = std::env::var("APP_PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .expect("APP_PORT must be a valid port number");

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;

    info!("Server listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
