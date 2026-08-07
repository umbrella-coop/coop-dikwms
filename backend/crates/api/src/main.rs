#![recursion_limit = "512"]

//! API server bootstrap: serves the platform REST + SSE surface.
//! Env: TERMINUSDB_URL (default http://localhost:6363), TERMINUSDB_DB (default
//! platform), TERMINUSDB_ADMIN_PASS (default root), PORT (default 8080).

use std::env;
use std::net::SocketAddr;

use schema_registry::SchemaRegistry;
use terminusdb_repository::Repository;
use url::Url;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let endpoint = env::var("TERMINUSDB_URL").unwrap_or_else(|_| "http://localhost:6363".into());
    let db = env::var("TERMINUSDB_DB").unwrap_or_else(|_| "platform".into());
    let pass = env::var("TERMINUSDB_ADMIN_PASS").unwrap_or_else(|_| "root".into());
    let port: u16 = env::var("PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(8080);

    let client = terminusdb_client::TerminusDBHttpClient::new(
        Url::parse(&endpoint)?,
        "admin",
        &pass,
        "admin",
    )
    .await?;
    let repo = Repository::new(client.clone(), db.clone()).await?;
    let registry = SchemaRegistry::init(client, format!("{db}_registry")).await?;

    let app = api::router(repo, registry);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("serving on {addr} (TerminusDB {endpoint}, db {db})");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
