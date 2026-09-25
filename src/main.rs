use golde_marketing_api::{app, config::Config, state::AppState};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize Tracing Subscriber
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,golde_marketing_api=debug,tower_http=info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 2. Load Configuration
    let config = Config::from_env().map_err(|e| format!("Configuration Error: {}", e))?;
    info!("🚀 Booting GOLD-e GrowthOS Marketing API [{}]", config.environment);

    // 3. Connect Database Pool
    info!(" Connecting to PostgreSQL via SQLx...");
    let pool = PgPoolOptions::new()
        .max_connections(25)
        .connect(&config.database_url)
        .await
        .map_err(|e| format!("Database connection error: {}", e))?;

    info!(" Database pool active. Running pending database migrations...");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| format!("Migration error: {}", e))?;
    info!(" Database migrations applied successfully.");

    // 4. Create App State and Router
    let state = AppState::new(pool, config.clone());
    let router = app(state.clone());

    // Native automation scheduler: durable schedules are stored in PostgreSQL.
    let scheduler_state = state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));
        loop {
            interval.tick().await;
            if let Err(e) = crate::services::automation_service::scheduler_tick(&scheduler_state).await {
                tracing::error!("Automation scheduler tick failed: {}", e);
            }
        }
    });

    // 5. Start HTTP Server
    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    info!(" Listening on http://{}", addr);
    info!(" Command Center Dashboard endpoint: http://{}/api/v1/analytics/dashboard", addr);
    info!(" Agent Runs endpoint: http://{}/api/v1/agents/runs", addr);
    info!(" WhatsApp Webhook endpoint: http://{}/api/v1/webhooks/whatsapp", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!(" Server shutdown completed.");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => info!("Received SIGINT, initiating graceful shutdown..."),
        _ = terminate => info!("Received SIGTERM, initiating graceful shutdown..."),
    }
}
