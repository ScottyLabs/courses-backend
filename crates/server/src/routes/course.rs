use std::collections::HashSet;

use crate::dtos::course::{
    ComponentResponse, CourseQueryParams, CourseResponse, MeetingResponse,
    PaginatedCoursesResponse, PaginationMeta,
};
use axum::{
    Json,
    extract::{Path, Query},
    http::StatusCode,
};
use database::{
    db::create_connection,
    entities::{components, courses, instructors, meetings},
    services::course::CourseService,
};
use sea_orm::{EntityTrait, QuerySelect, prelude::Uuid};
use serde_json::json;

/// Get paginated list of courses
#[utoipa::path(
    get,
    path = "/courses",
    params(CourseQueryParams),
    responses(
        (status = 200, description = "List of courses retrieved successfully", body = PaginatedCoursesResponse),
        (status = 400, description = "Invalid query parameters"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Courses"
)]
pub async fn get_courses(
    Query(params): Query<CourseQueryParams>,
) -> Result<Json<PaginatedCoursesResponse>, StatusCode> {
    let db = create_connection()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Query courses
    let (courses, total_items) = CourseService::get_courses_paginated(
        &db,
        params.page,
        params.per_page,
        params.season,
        params.year,
        params.search,
        params.department,
    )
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Get detailed course data with components
    let course_ids = courses.iter().map(|c| c.id).collect();
    let detailed_courses = CourseService::get_courses_with_components(&db, course_ids)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Convert to response DTOs
    let course_responses = detailed_courses
        .into_iter()
        .map(|(course, components)| convert_to_course_response(course, components))
        .collect();

    // Calculate pagination metadata
    let total_pages = total_items.div_ceil(params.per_page);
    let pagination = PaginationMeta {
        page: params.page,
        per_page: params.per_page,
        total_pages,
        total_items,
        has_next: params.page < total_pages,
        has_prev: params.page > 1,
    };

    Ok(Json(PaginatedCoursesResponse {
        courses: course_responses,
        pagination,
    }))
}

/// Get a specific course by ID
#[utoipa::path(
    get,
    path = "/courses/{id}",
    params(
        ("id" = Uuid, Path, description = "Course ID")
    ),
    responses(
        (status = 200, description = "Course found", body = CourseResponse),
        (status = 404, description = "Course not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Courses"
)]
pub async fn get_course_by_id(Path(id): Path<Uuid>) -> Result<Json<CourseResponse>, StatusCode> {
    // Create database connection
    let db = create_connection()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Get course by ID
    let course_data = CourseService::get_course_by_id(&db, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match course_data {
        Some((course, components)) => {
            let response = convert_to_course_response(course, components);
            Ok(Json(response))
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Get available seasons and years for filtering
#[utoipa::path(
    get,
    path = "/courses/filters",
    responses(
        (status = 200, description = "Filter options retrieved successfully"),
        (status = 500, description = "Internal server error")
    ),
    tag = "Courses"
)]
pub async fn get_course_filters() -> Result<Json<serde_json::Value>, StatusCode> {
    // Create database connection
    let db = create_connection()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Get distinct seasons and years
    let seasons_and_years = courses::Entity::find()
        .select_only()
        .column(courses::Column::Season)
        .column(courses::Column::Year)
        .distinct()
        .into_tuple::<(String, i16)>()
        .all(&db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut seasons = HashSet::new();
    let mut years = HashSet::new();

    for (season, year) in seasons_and_years {
        seasons.insert(season);
        years.insert(year);
    }

    let seasons_vec: Vec<_> = seasons.into_iter().collect();
    let mut years_vec: Vec<_> = years.into_iter().collect();

    years_vec.sort_by(|a, b| b.cmp(a)); // Sort years descending

    Ok(Json(json!({
        "seasons": seasons_vec,
        "years": years_vec,
    })))
}

type MeetingModel = (meetings::Model, Vec<instructors::Model>);
type ComponentModel = (components::Model, Vec<MeetingModel>);

/// Helper function to convert database models to API response
fn convert_to_course_response(
    course: courses::Model,
    components: Vec<ComponentModel>,
) -> CourseResponse {
    let related_urls: Vec<String> = course
        .related_urls
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_default();

    let component_responses: Vec<ComponentResponse> = components
        .into_iter()
        .map(|(component, meetings)| {
            let meeting_responses: Vec<MeetingResponse> = meetings
                .into_iter()
                .map(|(meeting, instructors)| {
                    let instructor_names: Vec<String> = instructors
                        .into_iter()
                        .map(|instructor| instructor.name)
                        .collect();

                    MeetingResponse {
                        id: meeting.id.to_string(),
                        days_pattern: meeting.days_pattern,
                        time_begin: meeting.time_begin,
                        time_end: meeting.time_end,
                        bldg_room: meeting.bldg_room,
                        campus: meeting.campus,
                        instructors: instructor_names,
                    }
                })
                .collect();

            ComponentResponse {
                id: component.id.to_string(),
                title: component.title,
                component_type: component.component_type,
                code: component.code,
                syllabus_url: component.syllabus_url,
                meetings: meeting_responses,
            }
        })
        .collect();

    CourseResponse {
        id: course.id.to_string(),
        number: course.number,
        units: course.units,
        season: course.season,
        year: course.year,
        special_permission: course.special_permission,
        description: course.description,
        prerequisites: course.prerequisites,
        notes: course.notes,
        related_urls,
        components: component_responses,
    }
}
