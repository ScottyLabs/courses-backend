use crate::routes::{auth, course, root};
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
    paths(
        root::root,
        auth::auth,
        course::get_courses,
        course::get_course_by_id,
        course::get_course_filters
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "Authentication", description = "Authentication related endpoints"),
        (name = "Courses", description = "Course related endpoints"),
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
