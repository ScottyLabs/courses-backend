mod routes;
mod utils;

use axum::{Json, routing::get};
use axum_server::{Handle, tls_rustls::RustlsConfig};
use dotenv_codegen::dotenv;
use log::info;
use routes::{health, root};
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_oauth2_resource_server::server::OAuth2ResourceServer;
use utils::shutdown::shutdown_signal;
use utoipa_axum::{router::OpenApiRouter, routes};

const OIDC_ISSUER_URL: &str = dotenv!("OIDC_ISSUER_URL");

#[tokio::main]
async fn main() {
    env_logger::init();

    let oauth2_resource_server = <OAuth2ResourceServer>::builder()
        .issuer_url(OIDC_ISSUER_URL)
        .build()
        .await
        .expect("Failed to build OAuth2ResourceServer");

    let protected_routes = OpenApiRouter::new()
        .routes(routes!(root::root))
        .layer(ServiceBuilder::new().layer(oauth2_resource_server.into_layer()));

    let public_routes = OpenApiRouter::new().routes(routes!(health::health));

    let (router, api) = OpenApiRouter::new()
        .merge(protected_routes)
        .merge(public_routes)
        .split_for_parts();

    // Add a route to serve the OpenAPI JSON
    let app = router.route("/openapi.json", get(|| async move { Json(api) }));

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
