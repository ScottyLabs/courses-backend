use crate::entities::{components, courses, instructor_meetings, instructors, meetings};
use models::{
    course_data::{ComponentType, CourseObject},
    syllabus_data::SyllabusMap,
};
use sea_orm::{ActiveValue::Set, DatabaseTransaction, TransactionTrait, prelude::*};
use std::collections::HashMap;

pub struct CourseService;

impl CourseService {
    /// The number of courses to save in a single batch
    const BATCH_SIZE: usize = 50;

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

        let mut all_course_ids = Vec::with_capacity(total_courses);

        // Process courses in batches
        for (batch_idx, batch) in course_objs.chunks(Self::BATCH_SIZE).enumerate() {
            let batch_start = batch_idx * Self::BATCH_SIZE;
            let batch_end = std::cmp::min(batch_start + Self::BATCH_SIZE, total_courses);

            println!(
                "Processing batch {}/{}: courses {}-{}",
                batch_idx + 1,
                total_courses.div_ceil(Self::BATCH_SIZE),
                batch_start + 1,
                batch_end
            );

            let batch_ids = Self::save_course_batch(db, batch.to_vec(), &syllabus_map).await?;
            all_course_ids.extend(batch_ids);

            println!(
                "Completed batch {}, {} courses processed so far",
                batch_idx + 1,
                all_course_ids.len()
            );
        }

        println!("Successfully saved all {total_courses} courses");
        Ok(all_course_ids)
    }

    async fn save_course_batch(
        db: &DatabaseConnection,
        course_objs: Vec<CourseObject>,
        syllabus_map: &SyllabusMap,
    ) -> Result<Vec<Uuid>, DbErr> {
        let txn = db.begin().await?;
        let mut course_ids = Vec::with_capacity(course_objs.len());

        // Pre-load all instructors to avoid N+1 queries
        let instructor_cache = Self::build_instructor_cache(&txn, &course_objs).await?;

        for (idx, course_obj) in course_objs.into_iter().enumerate() {
            if idx.is_multiple_of(10) {
                println!("  Saving course {} in current batch", idx + 1);
            }

            match Self::save_course(course_obj, syllabus_map, &txn, &instructor_cache).await {
                Ok(id) => course_ids.push(id),
                Err(e) => {
                    eprintln!("Error saving course: {e}");
                    txn.rollback().await.ok();
                    return Err(e);
                }
            }
        }

        txn.commit().await?;
        Ok(course_ids)
    }

    /// Build a cache of all instructors to avoid N+1 queries
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

    async fn save_course(
        course_obj: CourseObject,
        syllabus_map: &SyllabusMap,
        txn: &DatabaseTransaction,
        instructor_cache: &HashMap<String, Uuid>,
    ) -> Result<Uuid, DbErr> {
        let course_model = Self::course_to_active_model(&course_obj);
        let course_result = courses::Entity::insert(course_model).exec(txn).await?;
        let course_id = course_result.last_insert_id;

        // Save components for this course
        for component in course_obj.course.components {
            let key = (
                course_obj.course.year,
                course_obj.course.season,
                course_obj.course.number.to_string(),
                component.code.to_owned(),
            );
            let syllabus_url = syllabus_map.get(&key).cloned();

            let component_model = components::ActiveModel {
                id: Set(Uuid::new_v4()),
                course_id: Set(course_id),
                title: Set(component.title),
                component_type: Set(match component.component_type {
                    ComponentType::Lecture => "Lecture".to_string(),
                    ComponentType::Section => "Section".to_string(),
                }),
                code: Set(component.code),
                syllabus_url: Set(syllabus_url),
            };
            let component_result = components::Entity::insert(component_model)
                .exec(txn)
                .await?;
            let component_id = component_result.last_insert_id;

            // Save meetings for this component
            for meeting in component.meetings {
                let meeting_model = meetings::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    component_id: Set(component_id),
                    days_pattern: Set(meeting.days.to_string()),
                    time_begin: Set(meeting.time.as_ref().map(|t| t.begin)),
                    time_end: Set(meeting.time.as_ref().map(|t| t.end)),
                    bldg_room: Set(meeting.bldg_room.to_string()),
                    campus: Set(meeting.campus),
                };
                let meeting_result = meetings::Entity::insert(meeting_model).exec(txn).await?;
                let meeting_id = meeting_result.last_insert_id;

                // Save instructors for this meeting using the cache
                for instructor in meeting.instructors.as_ref().unwrap_or(&vec![]) {
                    let instructor_id = instructor_cache.get(instructor).ok_or_else(|| {
                        DbErr::Custom(format!("Instructor {instructor} not found in cache"))
                    })?;

                    // Create a many-to-many link between instructor and meeting
                    let link = instructor_meetings::ActiveModel {
                        id: Set(Uuid::new_v4()),
                        instructor_id: Set(*instructor_id),
                        meeting_id: Set(meeting_id),
                    };

                    instructor_meetings::Entity::insert(link).exec(txn).await?;
                }
            }
        }

        Ok(course_id)
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
}
