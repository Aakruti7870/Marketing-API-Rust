use golde_marketing_api::{app, config::Config, services::automation_service, state::AppState};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tokio::time::{interval, Duration};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,golde_marketing_api=debug,tower_http=info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config=Config::from_env().map_err(|e|format!("Configuration Error: {e}"))?;
    info!("Booting GOLD-e GrowthOS Marketing API [{}]",config.environment);

    let pool=PgPoolOptions::new().max_connections(25).connect(&config.database_url).await
        .map_err(|e|format!("Database connection error: {e}"))?;
    sqlx::migrate!("./migrations").run(&pool).await.map_err(|e|format!("Migration error: {e}"))?;

    let state=AppState::new(pool,config.clone());
    let scheduler_pool=state.pool.clone();
    let scheduler_http=state.http_client.clone();
    tokio::spawn(async move {
        let mut tick=interval(Duration::from_secs(15));
        loop {
            tick.tick().await;
            match automation_service::run_due_schedules(&scheduler_pool,&scheduler_http).await {
                Ok(n) if n>0 => info!(scheduled=n,"Automation scheduler executed due workflows"),
                Ok(_) => {},
                Err(e) => tracing::error!(error=%e,"Automation scheduler tick failed"),
            }
        }
    });

    let router=app(state);
    let addr:SocketAddr=format!("{}:{}",config.host,config.port).parse()?;
    info!("Listening on http://{addr}");
    let listener=tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener,router).with_graceful_shutdown(shutdown_signal()).await?;
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c=async{tokio::signal::ctrl_c().await.expect("failed to install Ctrl+C handler");};
    #[cfg(unix)]
    let terminate=async{tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).expect("failed to install signal handler").recv().await;};
    #[cfg(not(unix))]
    let terminate=std::future::pending::<()>();
    tokio::select!{_=ctrl_c=>info!("Received SIGINT, shutting down..."),_=terminate=>info!("Received SIGTERM, shutting down...")}
}