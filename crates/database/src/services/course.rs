use crate::entities::{components, courses, instructor_meetings, instructors, meetings};
use futures::future::try_join_all;
use models::{
    course_data::{ComponentType, CourseObject},
    syllabus_data::SyllabusMap,
};
use sea_orm::{
    ActiveValue::Set, ColumnTrait, Condition, DatabaseConnection, DatabaseTransaction, DbErr,
    EntityTrait, PaginatorTrait, QueryFilter, TransactionTrait,
};
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

pub struct CourseService;

impl CourseService {
    /// The number of courses to save in a single batch
    const BATCH_SIZE: usize = 200;

    pub async fn save_courses(
        db: &DatabaseConnection,
        course_objs: Vec<CourseObject>,
        syllabus_map: SyllabusMap,
    ) -> Result<Vec<Uuid>, DbErr> {
        let total_courses = course_objs.len();
        println!(
            "Starting to save {} courses in batches of {}",
            total_courses,
            Self::BATCH_SIZE
        );

        let syllabus_map = Arc::new(syllabus_map);

        let batch_futures =
            course_objs
                .chunks(Self::BATCH_SIZE)
                .enumerate()
                .map(|(batch_idx, batch)| {
                    let db = db.clone();
                    let syllabus_map = Arc::clone(&syllabus_map);
                    let batch_vec = batch.to_vec();

                    async move {
                        let batch_start = batch_idx * Self::BATCH_SIZE;
                        let batch_end =
                            std::cmp::min(batch_start + Self::BATCH_SIZE, total_courses);

                        println!(
                            "Processing batch {}/{}: courses {}-{}",
                            batch_idx + 1,
                            total_courses.div_ceil(Self::BATCH_SIZE),
                            batch_start + 1,
                            batch_end
                        );

                        let result = Self::save_course_batch(&db, batch_vec, &syllabus_map).await;

                        match &result {
                            Ok(ids) => println!(
                                "Completed batch {}, {} courses processed",
                                batch_idx + 1,
                                ids.len()
                            ),
                            Err(e) => eprintln!("Error in batch {}: {}", batch_idx + 1, e),
                        }

                        result
                    }
                });

        let all_batch_results: Vec<Vec<Uuid>> = try_join_all(batch_futures).await?;
        let all_course_ids = all_batch_results.into_iter().flatten().collect();

        println!("Successfully saved all {total_courses} courses");
        Ok(all_course_ids)
    }

    async fn save_course_batch(
        db: &DatabaseConnection,
        course_objs: Vec<CourseObject>,
        syllabus_map: &SyllabusMap,
    ) -> Result<Vec<Uuid>, DbErr> {
        let txn = db.begin().await?;
        let instructor_cache = Self::build_instructor_cache(&txn, &course_objs).await?;

        // Collect all data for bulk insertion
        let mut all_courses = Vec::new();
        let mut all_components = Vec::new();
        let mut all_meetings = Vec::new();
        let mut all_instructor_meetings = Vec::new();
        let mut course_ids = Vec::new();

        // Prepare all data first
        for (idx, course_obj) in course_objs.into_iter().enumerate() {
            if idx.is_multiple_of(10) {
                println!("  Saving course {idx} in current batch");
            }

            let course_id = Uuid::new_v4();
            course_ids.push(course_id);

            // Prepare course
            all_courses.push(Self::course_to_active_model(&course_obj));

            // Prepare components for this course
            for component in course_obj.course.components {
                let component_id = Uuid::new_v4();

                let key = (
                    course_obj.course.year,
                    course_obj.course.season,
                    course_obj.course.number.to_string(),
                    component.code.clone(),
                );
                let syllabus_url = syllabus_map.get(&key).cloned();

                all_components.push(components::ActiveModel {
                    id: Set(component_id),
                    course_id: Set(course_id),
                    title: Set(component.title),
                    component_type: Set(match component.component_type {
                        ComponentType::Lecture => "Lecture".to_string(),
                        ComponentType::Section => "Section".to_string(),
                    }),
                    code: Set(component.code),
                    syllabus_url: Set(syllabus_url),
                });

                // Prepare meetings for this component
                for meeting in component.meetings {
                    let meeting_id = Uuid::new_v4();

                    all_meetings.push(meetings::ActiveModel {
                        id: Set(meeting_id),
                        component_id: Set(component_id),
                        days_pattern: Set(meeting.days.to_string()),
                        time_begin: Set(meeting.time.as_ref().map(|t| t.begin)),
                        time_end: Set(meeting.time.as_ref().map(|t| t.end)),
                        bldg_room: Set(meeting.bldg_room.to_string()),
                        campus: Set(meeting.campus),
                    });

                    // Save instructors for this meeting using the cache
                    if let Some(instructors) = meeting.instructors.as_ref() {
                        for instructor_name in instructors {
                            if let Some(&instructor_id) = instructor_cache.get(instructor_name) {
                                // Create a many-to-many link between instructor and meeting
                                all_instructor_meetings.push(instructor_meetings::ActiveModel {
                                    id: Set(Uuid::new_v4()),
                                    instructor_id: Set(instructor_id),
                                    meeting_id: Set(meeting_id),
                                });
                            }
                        }
                    }
                }
            }
        }

        // Bulk insert everything at once
        if !all_courses.is_empty() {
            courses::Entity::insert_many(all_courses).exec(&txn).await?;
        }
        if !all_components.is_empty() {
            components::Entity::insert_many(all_components)
                .exec(&txn)
                .await?;
        }
        if !all_meetings.is_empty() {
            meetings::Entity::insert_many(all_meetings)
                .exec(&txn)
                .await?;
        }
        if !all_instructor_meetings.is_empty() {
            instructor_meetings::Entity::insert_many(all_instructor_meetings)
                .exec(&txn)
                .await?;
        }

        txn.commit().await?;
        Ok(course_ids)
    }

    /// Build a cache of all instructors
    async fn build_instructor_cache(
        txn: &DatabaseTransaction,
        course_objs: &[CourseObject],
    ) -> Result<HashMap<String, Uuid>, DbErr> {
        // Collect all unique instructor names
        let mut instructor_names = std::collections::HashSet::new();

        for course_obj in course_objs {
            for component in &course_obj.course.components {
                for meeting in &component.meetings {
                    if let Some(instructors) = meeting.instructors.as_ref() {
                        for instructor in instructors {
                            instructor_names.insert(instructor.clone());
                        }
                    }
                }
            }
        }

        println!(
            "  Building cache for {} unique instructors",
            instructor_names.len()
        );

        // Fetch existing instructors
        let existing_instructors: Vec<instructors::Model> = instructors::Entity::find()
            .filter(instructors::Column::Name.is_in(instructor_names.clone()))
            .all(txn)
            .await?;

        let mut cache = HashMap::new();
        for instructor in existing_instructors {
            cache.insert(instructor.name.clone(), instructor.id);
            instructor_names.remove(&instructor.name);
        }

        // Create new instructors for ones that don't exist
        for new_instructor_name in instructor_names {
            let new_id = Uuid::new_v4();
            let new_instructor = instructors::ActiveModel {
                id: Set(new_id),
                name: Set(new_instructor_name.clone()),
            };

            instructors::Entity::insert(new_instructor)
                .exec(txn)
                .await?;

            cache.insert(new_instructor_name, new_id);
        }

        Ok(cache)
    }

    fn course_to_active_model(course_obj: &CourseObject) -> courses::ActiveModel {
        courses::ActiveModel {
            id: Set(Uuid::new_v4()),
            number: Set(course_obj.course.number.to_string()),
            units: Set(course_obj.course.units.to_string()),
            season: Set(course_obj.course.season.as_str().to_owned()),
            year: Set(*course_obj.course.year as i16),

            // When metadata is None, default to empty vec of related URLs
            related_urls: Set(course_obj
                .metadata
                .as_ref()
                .map(|m| m.related_urls.to_owned().into())
                .unwrap_or_default()),

            // When metadata is None, default no special permission
            special_permission: Set(course_obj
                .metadata
                .as_ref()
                .map(|m| m.special_permission)
                .unwrap_or(false)),

            // Description and prerequisites can be None
            description: Set(course_obj
                .metadata
                .as_ref()
                .and_then(|m| m.description.to_owned())),
            prerequisites: Set(course_obj
                .metadata
                .as_ref()
                .and_then(|m| m.prerequisites.clone().into_inner())
                .and_then(|expr| serde_json::to_string(&expr).ok())), // Handle serialization errors gracefully

            // For corequisites and crosslisted, use the same behavior as related URLs
            corequisites: Set(course_obj
                .metadata
                .as_ref()
                .map(|m| m.corequisites.to_vec().into())
                .unwrap_or_default()),
            crosslisted: Set(course_obj
                .metadata
                .as_ref()
                .map(|m| m.crosslisted.to_vec().into())
                .unwrap_or_default()),

            // Notes can also be None
            notes: Set(course_obj
                .metadata
                .as_ref()
                .and_then(|m| m.notes.to_owned())),
        }
    }

    /// Query courses with pagination and filtering
    pub async fn get_courses_paginated(
        db: &DatabaseConnection,
        page: u64,
        per_page: u64,
        seasons: Option<Vec<String>>,
        years: Option<Vec<i16>>,
        search: Option<String>,
        departments: Option<Vec<String>>,
    ) -> Result<(Vec<courses::Model>, u64), DbErr> {
        let mut query = courses::Entity::find();
        let mut condition = Condition::all();

        if let Some(seasons) = seasons
            && !seasons.is_empty()
        {
            condition = condition.add(courses::Column::Season.is_in(seasons));
        }

        if let Some(years) = years
            && !years.is_empty()
        {
            condition = condition.add(courses::Column::Year.is_in(years));
        }

        if let Some(search) = search {
            let search_condition = Condition::any()
                .add(courses::Column::Number.like(format!("%{search}%")))
                .add(courses::Column::Description.like(format!("%{search}%")));
            condition = condition.add(search_condition);
        }

        // The first two digits of the course number are the department prefix
        if let Some(departments) = departments
            && !departments.is_empty()
        {
            let mut dept_condition = Condition::any(); // Use OR
            for dept in &departments {
                let pattern = format!("{dept}%");
                dept_condition = dept_condition.add(courses::Column::Number.like(pattern));
            }
            condition = condition.add(dept_condition);
        }

        query = query.filter(condition);

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
