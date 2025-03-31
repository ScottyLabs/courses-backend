use std::net::SocketAddr;

use axum::{Extension, Router, http::StatusCode, routing::get};
use axum_server::{Handle, tls_rustls::RustlsConfig};
use dotenv_codegen::dotenv;
use log::info;
use tokio::signal;
use tower::ServiceBuilder;
use tower_oauth2_resource_server::{claims::DefaultClaims, server::OAuth2ResourceServer};

const OIDC_ISSUER_URL: &str = dotenv!("OIDC_ISSUER_URL");

async fn health() -> &'static str {
    "OK"
}

async fn root(claims: Extension<DefaultClaims>) -> Result<(StatusCode, String), StatusCode> {
    let sub = claims
        .sub
        .as_ref()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::OK, format!("Hello, {}", sub)))
}

async fn shutdown_signal(handle: Handle) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    // Trigger graceful shutdown
    handle.graceful_shutdown(None);
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let oauth2_resource_server = <OAuth2ResourceServer>::builder()
        .issuer_url(OIDC_ISSUER_URL)
        .build()
        .await
        .expect("Failed to build OAuth2ResourceServer");

    let protected_routes = Router::new()
        .route("/", get(root))
        .layer(ServiceBuilder::new().layer(oauth2_resource_server.into_layer()));

    let public_routes = Router::new().route("/health", get(health));

    let app = Router::new().merge(protected_routes).merge(public_routes);

    // Load TLS configuration
    let cert_path = std::env::var("TLS_CERT_PATH").unwrap_or("certs/localhost+2.pem".to_string());
    let key_path = std::env::var("TLS_KEY_PATH").unwrap_or("certs/localhost+2-key.pem".to_string());

    let config = RustlsConfig::from_pem_file(cert_path, key_path)
        .await
        .expect("Failed to load TLS configuration");

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("Running axum on https://localhost:3000");

    // Create server handle for graceful shutdown
    let handle = Handle::new();
    let shutdown_handle = handle.clone();
    tokio::spawn(shutdown_signal(shutdown_handle));

    // Bind with TLS configuration
    axum_server::bind_rustls(addr, config)
        .handle(handle) // axum_server does not support axum's .with_graceful_shutdown
        .serve(app.into_make_service())
        .await
        .unwrap();
}
