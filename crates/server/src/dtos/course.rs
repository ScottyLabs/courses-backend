use chrono::NaiveTime;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Serialize, ToSchema)]
pub struct CourseResponse {
    pub id: String,
    pub number: String,
    pub units: String,
    pub season: String,
    pub year: i16,
    pub special_permission: bool,
    pub description: Option<String>,
    pub prerequisites: Option<String>,
    pub notes: Option<String>,
    pub related_urls: Vec<String>,
    pub components: Vec<ComponentResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ComponentResponse {
    pub id: String,
    pub title: String,
    pub component_type: String,
    pub code: String,
    pub syllabus_url: Option<String>,
    pub meetings: Vec<MeetingResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MeetingResponse {
    pub id: String,
    pub days_pattern: String,
    pub time_begin: Option<NaiveTime>,
    pub time_end: Option<NaiveTime>,
    pub bldg_room: String,
    pub campus: String,
    pub instructors: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginatedCoursesResponse {
    pub courses: Vec<CourseResponse>,
    pub pagination: PaginationMeta,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaginationMeta {
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
    pub total_items: u64,
    pub has_next: bool,
    pub has_prev: bool,
}

#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct CourseQueryParams {
    #[serde(default = "default_page")]
    pub page: u64,

    #[serde(default = "default_per_page")]
    pub per_page: u64,

    pub season: Option<Vec<String>>,
    pub year: Option<Vec<i16>>,
    pub search: Option<String>,
    pub department: Option<Vec<String>>,
}

fn default_page() -> u64 {
    1
}

fn default_per_page() -> u64 {
    20
}
