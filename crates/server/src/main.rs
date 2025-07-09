mod doc;
mod routes;
mod utils;

use doc::ApiDoc;
use log::info;
use routes::{auth, root};
use tower::ServiceBuilder;
use tower_oauth2_resource_server::server::OAuth2ResourceServer;
use utils::shutdown::shutdown_signal;
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() {
    env_logger::init();
    dotenvy::dotenv().ok();

    let oidc_issuer_url =
        std::env::var("OIDC_ISSUER_URL").expect("OIDC_ISSUER_URL environment variable must be set");

    let oauth2_resource_server = <OAuth2ResourceServer>::builder()
        .issuer_url(oidc_issuer_url)
        .build()
        .await
        .expect("Failed to build OAuth2ResourceServer");

    let protected_routes = OpenApiRouter::new()
        .routes(routes!(auth::auth))
        .layer(ServiceBuilder::new().layer(oauth2_resource_server.into_layer()));

    let public_routes = OpenApiRouter::new().routes(routes!(root::root));

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
