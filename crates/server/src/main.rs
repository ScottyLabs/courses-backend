mod doc;
mod routes;
mod utils;

use doc::ApiDoc;
use dotenv_codegen::dotenv;
use log::info;
use routes::{auth, health};
use tower::ServiceBuilder;
use tower_oauth2_resource_server::server::OAuth2ResourceServer;
use utils::shutdown::shutdown_signal;
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_swagger_ui::SwaggerUi;

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
        .routes(routes!(auth::root))
        .layer(ServiceBuilder::new().layer(oauth2_resource_server.into_layer()));

    let public_routes = OpenApiRouter::new().routes(routes!(health::health));

    let (router, _api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .merge(protected_routes)
        .merge(public_routes)
        .split_for_parts();

    let app = router.merge(SwaggerUi::new("/swagger").url("/openapi.json", ApiDoc::openapi()));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    info!("Running axum on http://localhost:3000");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}
