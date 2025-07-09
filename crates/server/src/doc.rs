use crate::routes::{auth, root};
use utoipa::{
    Modify, OpenApi,
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
};

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.as_mut().unwrap();
        components.add_security_scheme(
            "jwt",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );
    }
}

/// API Documentation
#[derive(OpenApi)]
#[openapi(
    paths(root::root, auth::auth),
    modifiers(&SecurityAddon),
    tags(
        (name = "Authentication", description = "Authentication related endpoints"),
    ),
    info(
        title = "Course API",
        version = "1.0.0",
        description = "CMU Courses API",
        license(
            name = "MIT OR Apache-2.0",
        )
    )
)]
pub struct ApiDoc;
