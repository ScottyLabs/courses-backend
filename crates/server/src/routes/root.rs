use axum::http::StatusCode;

/// Simple endpoint that returns "OK" when the service is running properly
#[utoipa::path(
    get,
    path = "/",
    responses(
        (status = 200, description = "Service is healthy", content_type = "text/plain", body = String)
    ),
    tag = ""
)]
pub async fn root() -> (StatusCode, &'static str) {
    (StatusCode::OK, "OK")
}
