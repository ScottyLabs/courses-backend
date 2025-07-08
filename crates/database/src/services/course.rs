use crate::entities::{component, course, instructor, instructor_meeting, meeting};
use chrono::{NaiveDateTime, Utc};
use models::course_data::CourseObject;
use sea_orm::{ActiveValue::Set, TransactionTrait, prelude::*};

pub struct CourseService;

impl CourseService {
    pub async fn save_course(
        db: &DatabaseConnection,
        course_obj: CourseObject,
    ) -> Result<Uuid, DbErr> {
        let txn = db.begin().await?;
        let time = Utc::now().naive_utc();

        let course_model = Self::course_to_active_model(&course_obj, time);
        let course_result = course::Entity::insert(course_model).exec(&txn).await?;
        let course_id = course_result.last_insert_id;

        // Save components for this course
        for component in course_obj.course.components {
            let component_model = component::ActiveModel {
                id: Set(Uuid::new_v4()),
                course_id: Set(course_id),
                title: Set(component.title),
                component_type: Set(component.component_type),
                code: Set(component.code),
                created_at: Set(time),
                updated_at: Set(time),
            };
            let component_result = component::Entity::insert(component_model)
                .exec(&txn)
                .await?;
            let component_id = component_result.last_insert_id;

            // Save meetings for this component
            for meeting in component.meetings {
                let meeting_model = meeting::ActiveModel {
                    id: Set(Uuid::new_v4()),
                    component_id: Set(component_id),
                    days_pattern: Set(meeting.days.to_string()),
                    time_begin: Set(meeting.time.as_ref().map(|t| t.begin)),
                    time_end: Set(meeting.time.as_ref().map(|t| t.end)),
                    bldg_room: Set(meeting.bldg_room.to_string()),
                    campus: Set(meeting.campus),
                    created_at: Set(time),
                    updated_at: Set(time),
                };
                let meeting_result = meeting::Entity::insert(meeting_model).exec(&txn).await?;
                let meeting_id = meeting_result.last_insert_id;

                // Save instructors for this meeting
                for instructor in meeting.instructors.as_ref().unwrap_or(&vec![]) {
                    // Check if instructor already exists by name
                    let existing = instructor::Entity::find()
                        .filter(instructor::Column::Name.eq(instructor))
                        .one(&txn)
                        .await?;

                    let instructor_id = match existing {
                        Some(model) => model.id,
                        None => {
                            // Insert new instructor
                            let new_id = Uuid::new_v4();
                            let new_instructor = instructor::ActiveModel {
                                id: Set(new_id),
                                name: Set(instructor.to_owned()),
                                created_at: Set(time),
                                updated_at: Set(time),
                            };
                            instructor::Entity::insert(new_instructor)
                                .exec(&txn)
                                .await?;

                            new_id
                        }
                    };

                    // Create a many-to-many link between instructor and meeting
                    let link = instructor_meeting::ActiveModel {
                        id: Set(Uuid::new_v4()),
                        instructor_id: Set(instructor_id),
                        meeting_id: Set(meeting_id),
                        created_at: Set(time),
                    };

                    instructor_meeting::Entity::insert(link).exec(&txn).await?;
                }
            }
        }

        txn.commit().await?;
        Ok(course_id)
    }

    fn course_to_active_model(
        course_obj: &CourseObject,
        time: NaiveDateTime,
    ) -> course::ActiveModel {
        course::ActiveModel {
            id: Set(Uuid::new_v4()),
            number: Set(course_obj.course.number.to_string()),
            units: Set(course_obj.course.units.to_string()),
            season: Set(course_obj.course.season.as_str().to_owned()),
            year: Set(*course_obj.course.year),

            // When metadata is None, default to empty vec of related URLs
            related_urls: Set(course_obj
                .metadata
                .as_ref()
                .map(|m| m.related_urls.to_owned())
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
                .and_then(|m| m.prerequisites.clone().into_inner())),

            // For corequisites and crosslisted, use the same behavior as related URLs
            corequisites: Set(course_obj
                .metadata
                .as_ref()
                .map(|m| m.corequisites.to_vec())
                .unwrap_or_default()),
            crosslisted: Set(course_obj
                .metadata
                .as_ref()
                .map(|m| m.crosslisted.to_vec())
                .unwrap_or_default()),

            // Notes can also be None
            notes: Set(course_obj
                .metadata
                .as_ref()
                .and_then(|m| m.notes.to_owned())),

            created_at: Set(time),
            updated_at: Set(time),
        }
    }
}
