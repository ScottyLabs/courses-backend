use crate::entities::{components, courses, instructor_meetings, instructors, meetings};
use sea_orm::{
    ColumnTrait, Condition, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait, QueryFilter,
    QueryTrait, prelude::Expr, sea_query::ExprTrait,
};
use std::collections::HashMap;
use uuid::Uuid;

pub struct QueryCourseService;

impl QueryCourseService {
    /// Query courses with pagination and filtering
    pub async fn get_courses_paginated(
        db: &DatabaseConnection,
        page: u64,
        per_page: u64,
        seasons: Option<Vec<String>>,
        years: Option<Vec<i16>>,
        departments: Option<Vec<String>>,
    ) -> Result<(Vec<courses::Model>, u64), DbErr> {
        let mut base_condition = Condition::all();

        if let Some(seasons) = seasons
            && !seasons.is_empty()
        {
            base_condition = base_condition.add(courses::Column::Season.is_in(seasons));
        }

        if let Some(years) = years
            && !years.is_empty()
        {
            base_condition = base_condition.add(courses::Column::Year.is_in(years));
        }

        // The first two digits of the course number are the department prefix
        if let Some(departments) = departments
            && !departments.is_empty()
        {
            let dept_codes: Vec<String> = departments.into_iter().collect();
            base_condition = base_condition
                .add(Expr::cust("substring(courses.number from 1 for 2)").is_in(dept_codes));
        }

        let query = courses::Entity::find().filter(base_condition);

        println!(
            "Generated SQL: {}",
            query.build(sea_orm::DatabaseBackend::Postgres)
        );

        // Apply pagination
        let total_items = query.clone().count(db).await?;
        let paginator = query.paginate(db, per_page);
        let courses = paginator.fetch_page(page - 1).await?; // SeaORM uses 0-based pages

        Ok((courses, total_items))
    }

    /// Get a single course with all its components, meetings, and instructors
    pub async fn get_course_by_id(
        db: &DatabaseConnection,
        course_id: Uuid,
    ) -> Result<
        Option<(
            courses::Model,
            Vec<(
                components::Model,
                Vec<(meetings::Model, Vec<instructors::Model>)>,
            )>,
        )>,
        DbErr,
    > {
        let course = match courses::Entity::find_by_id(course_id).one(db).await? {
            Some(course) => course,
            None => return Ok(None),
        };

        // Get all components for this course
        let components = components::Entity::find()
            .filter(components::Column::CourseId.eq(course_id))
            .all(db)
            .await?;

        if components.is_empty() {
            return Ok(Some((course, vec![])));
        }

        let component_ids: Vec<Uuid> = components.iter().map(|c| c.id).collect();

        // Batch fetch all meetings for all components
        let meetings = meetings::Entity::find()
            .filter(meetings::Column::ComponentId.is_in(component_ids))
            .all(db)
            .await?;

        if meetings.is_empty() {
            let result_components = components
                .into_iter()
                .map(|component| (component, vec![]))
                .collect();
            return Ok(Some((course, result_components)));
        }

        let meeting_ids: Vec<Uuid> = meetings.iter().map(|m| m.id).collect();

        // Batch fetch all instructor-meeting relationships
        let instructor_meetings: Vec<(instructor_meetings::Model, instructors::Model)> =
            instructor_meetings::Entity::find()
                .filter(instructor_meetings::Column::MeetingId.is_in(meeting_ids))
                .find_also_related(instructors::Entity)
                .all(db)
                .await?
                .into_iter()
                .filter_map(|(im, instructor)| instructor.map(|i| (im, i)))
                .collect();

        // Build lookup maps
        let mut meetings_by_component: HashMap<Uuid, Vec<meetings::Model>> = HashMap::new();
        for meeting in meetings {
            meetings_by_component
                .entry(meeting.component_id)
                .or_default()
                .push(meeting);
        }

        let mut instructors_by_meeting: HashMap<Uuid, Vec<instructors::Model>> = HashMap::new();
        for (im, instructor) in instructor_meetings {
            instructors_by_meeting
                .entry(im.meeting_id)
                .or_default()
                .push(instructor);
        }

        // Build the final result structure
        let mut result_components = Vec::new();
        for component in components {
            let component_meetings = meetings_by_component
                .remove(&component.id)
                .unwrap_or_default();

            let mut result_meetings = Vec::new();
            for meeting in component_meetings {
                let meeting_instructors = instructors_by_meeting
                    .remove(&meeting.id)
                    .unwrap_or_default();
                result_meetings.push((meeting, meeting_instructors));
            }

            result_components.push((component, result_meetings));
        }

        Ok(Some((course, result_components)))
    }

    /// Get multiple courses with their components (for list view)
    pub async fn get_courses_with_components(
        db: &DatabaseConnection,
        course_ids: Vec<Uuid>,
    ) -> Result<
        Vec<(
            courses::Model,
            Vec<(
                components::Model,
                Vec<(meetings::Model, Vec<instructors::Model>)>,
            )>,
        )>,
        DbErr,
    > {
        if course_ids.is_empty() {
            return Ok(vec![]);
        }

        // Batch fetch all courses
        let courses = courses::Entity::find()
            .filter(courses::Column::Id.is_in(course_ids.clone()))
            .all(db)
            .await?;

        // Batch fetch all components for all courses
        let components = components::Entity::find()
            .filter(components::Column::CourseId.is_in(course_ids))
            .all(db)
            .await?;

        if components.is_empty() {
            let results = courses.into_iter().map(|course| (course, vec![])).collect();
            return Ok(results);
        }

        let component_ids: Vec<Uuid> = components.iter().map(|c| c.id).collect();

        // Batch fetch all meetings for all components
        let meetings = meetings::Entity::find()
            .filter(meetings::Column::ComponentId.is_in(component_ids))
            .all(db)
            .await?;

        if meetings.is_empty() {
            let mut components_by_course: HashMap<Uuid, Vec<components::Model>> = HashMap::new();
            for component in components {
                components_by_course
                    .entry(component.course_id)
                    .or_default()
                    .push(component);
            }

            let results = courses
                .into_iter()
                .map(|course| {
                    let course_components = components_by_course
                        .remove(&course.id)
                        .unwrap_or_default()
                        .into_iter()
                        .map(|component| (component, vec![]))
                        .collect();
                    (course, course_components)
                })
                .collect();
            return Ok(results);
        }

        let meeting_ids: Vec<Uuid> = meetings.iter().map(|m| m.id).collect();

        // Batch fetch all instructor-meeting relationships
        let instructor_meetings: Vec<(instructor_meetings::Model, instructors::Model)> =
            instructor_meetings::Entity::find()
                .filter(instructor_meetings::Column::MeetingId.is_in(meeting_ids))
                .find_also_related(instructors::Entity)
                .all(db)
                .await?
                .into_iter()
                .filter_map(|(im, instructor)| instructor.map(|i| (im, i)))
                .collect();

        // Build lookup maps
        let mut components_by_course: HashMap<Uuid, Vec<components::Model>> = HashMap::new();
        for component in components {
            components_by_course
                .entry(component.course_id)
                .or_default()
                .push(component);
        }

        let mut meetings_by_component: HashMap<Uuid, Vec<meetings::Model>> = HashMap::new();
        for meeting in meetings {
            meetings_by_component
                .entry(meeting.component_id)
                .or_default()
                .push(meeting);
        }

        let mut instructors_by_meeting: HashMap<Uuid, Vec<instructors::Model>> = HashMap::new();
        for (im, instructor) in instructor_meetings {
            instructors_by_meeting
                .entry(im.meeting_id)
                .or_default()
                .push(instructor);
        }

        // Build the final result structure
        let mut results = Vec::new();
        for course in courses {
            let course_components = components_by_course.remove(&course.id).unwrap_or_default();

            let mut result_components = Vec::new();
            for component in course_components {
                let component_meetings = meetings_by_component
                    .remove(&component.id)
                    .unwrap_or_default();

                let mut result_meetings = Vec::new();
                for meeting in component_meetings {
                    let meeting_instructors = instructors_by_meeting
                        .remove(&meeting.id)
                        .unwrap_or_default();
                    result_meetings.push((meeting, meeting_instructors));
                }

                result_components.push((component, result_meetings));
            }

            results.push((course, result_components));
        }

        Ok(results)
    }
}
