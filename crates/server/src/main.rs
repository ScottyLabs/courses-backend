use axum::{Extension, Router, http::StatusCode, routing::get};
use dotenv_codegen::dotenv;
use log::info;
use tokio::signal;
use tower::ServiceBuilder;
use tower_oauth2_resource_server::{claims::DefaultClaims, server::OAuth2ResourceServer};

const OIDC_ISSUER_URL: &str = dotenv!("OIDC_ISSUER_URL");

async fn root(claims: Extension<DefaultClaims>) -> Result<(StatusCode, String), StatusCode> {
    let sub = claims
        .sub
        .as_ref()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::OK, format!("Hello, {}", sub)))
}

async fn shutdown_signal() {
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
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let oauth2_resource_server = <OAuth2ResourceServer>::builder()
        // .audiences(&["courses-backend"])
        .issuer_url(OIDC_ISSUER_URL)
        .build()
        .await
        .expect("Failed to build OAuth2ResourceServer");

    let app = Router::new()
        .route("/", get(root))
        .layer(ServiceBuilder::new().layer(oauth2_resource_server.into_layer()));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    info!("Running axum on https://localhost:3000");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}
