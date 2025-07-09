use crate::entities::{components, courses, instructor_meetings, instructors, meetings};
use models::{
    course_data::{ComponentType, CourseObject},
    syllabus_data::SyllabusMap,
};
use sea_orm::{ActiveValue::Set, DatabaseTransaction, TransactionTrait, prelude::*};

pub struct CourseService;

impl CourseService {
    pub async fn save_courses(
        db: &DatabaseConnection,
        course_objs: Vec<CourseObject>,
        syllabus_map: SyllabusMap,
    ) -> Result<Vec<Uuid>, DbErr> {
        let txn = db.begin().await?;
        let mut course_ids = Vec::with_capacity(course_objs.len());

        for course_obj in course_objs {
            match Self::save_course(course_obj, &syllabus_map, &txn).await {
                Ok(id) => course_ids.push(id),
                Err(e) => {
                    txn.rollback().await.ok();
                    return Err(e);
                }
            }
        }

        txn.commit().await?;
        Ok(course_ids)
    }

    async fn save_course(
        course_obj: CourseObject,
        syllabus_map: &SyllabusMap,
        txn: &DatabaseTransaction,
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

                // Save instructors for this meeting
                for instructor in meeting.instructors.as_ref().unwrap_or(&vec![]) {
                    // Check if instructor already exists by name
                    let existing = instructors::Entity::find()
                        .filter(instructors::Column::Name.eq(instructor))
                        .one(txn)
                        .await?;

                    let instructor_id = match existing {
                        Some(model) => model.id,
                        None => {
                            // Insert new instructor
                            let new_id = Uuid::new_v4();
                            let new_instructor = instructors::ActiveModel {
                                id: Set(new_id),
                                name: Set(instructor.to_owned()),
                            };
                            instructors::Entity::insert(new_instructor)
                                .exec(txn)
                                .await?;

                            new_id
                        }
                    };

                    // Create a many-to-many link between instructor and meeting
                    let link = instructor_meetings::ActiveModel {
                        id: Set(Uuid::new_v4()),
                        instructor_id: Set(instructor_id),
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
                .map(|expr| serde_json::to_string(&expr).unwrap())),

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
