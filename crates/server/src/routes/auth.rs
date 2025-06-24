use axum::{Extension, http::StatusCode};
use tower_oauth2_resource_server::claims::DefaultClaims;

/// Returns a greeting with the user's subject identifier from their JWT claims
#[utoipa::path(
    get,
    path = "/",
    responses(
        (status = 200, description = "Successfully authenticated", content_type = "text/plain", body = String),
        (status = 401, description = "Unauthorized - invalid or missing JWT"),
        (status = 500, description = "Internal server error - missing subject in claims")
    ),
    security(
        ("jwt" = [])
    ),
    tag = "Authentication"
)]
pub async fn root(claims: Extension<DefaultClaims>) -> Result<(StatusCode, String), StatusCode> {
    let sub = claims
        .sub
        .as_ref()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::OK, format!("Hello, {sub}")))
}
