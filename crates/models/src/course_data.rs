use crate::{
    days::Days,
    requisite::{Prerequisites, Requisites},
    reservation::Reservation,
    syllabus_data::{Season, Year},
    units::Units,
};
use chrono::NaiveTime;
use sea_orm::EnumIter;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    ops::Deref,
    str::FromStr,
};
use strum::{EnumProperty, IntoEnumIterator};

#[cfg(feature = "database")]
use sea_orm::DeriveActiveEnum;

/// Represents a time range for a meeting
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct TimeRange {
    pub begin: NaiveTime,
    pub end: NaiveTime,
}

impl TimeRange {
    /// Creates a new [`TimeRange`] if `begin` is before `end`
    pub fn new(begin: NaiveTime, end: NaiveTime) -> Option<Self> {
        (begin < end).then_some(Self { begin, end })
    }

    /// Parses two time strings and creates a [`TimeRange`] if valid.
    /// # Returns
    /// `Some(TimeRange)` if parsing succeeds and `begin` is before `end`
    pub fn from_strings(begin: &str, end: &str) -> Option<Self> {
        let fmt = "%I:%M%p"; // 12-hour format with AM/PM
        let begin = NaiveTime::parse_from_str(begin, fmt).ok()?;
        let end = NaiveTime::parse_from_str(end, fmt).ok()?;

        Self::new(begin, end)
    }
}

/// Represents a place where a meeting can occur
#[derive(Debug, Clone, PartialEq, Serialize, EnumProperty, EnumIter)]
pub enum BuildingRoom {
    #[strum(props(display = "TBA", parse = "TBA"))]
    ToBeAnnounced,

    #[strum(props(display = "TBD", parse = "TBD TBD"))]
    ToBeDetermined,

    #[strum(props(display = "DNM", parse = "DNM DNM"))]
    DoesNotMeet,

    #[strum(props(display = "OFF PITT", parse = "OFF PITT"))]
    OffPitt,

    #[strum(props(display = "REMOTE", parse = "CMU REMOTE"))]
    Remote,

    Specific(String, String),
}

impl FromStr for BuildingRoom {
    type Err = ();

    fn from_str(bldg_room: &str) -> Result<Self, Self::Err> {
        Self::iter()
            .find(|v| {
                v.get_str("parse") == Some(bldg_room) || v.get_str("display") == Some(bldg_room)
            })
            .or_else(|| {
                // Split into building and room, defaulting to empty string if part is missing
                let mut parts = bldg_room.split_whitespace();
                Some(Self::Specific(
                    parts.next().unwrap_or("").to_string(),
                    parts.collect::<Vec<_>>().join(" "),
                ))
            })
            .ok_or(())
    }
}

impl From<String> for BuildingRoom {
    fn from(bldg_room: String) -> Self {
        Self::from_str(&bldg_room).unwrap()
    }
}

impl Display for BuildingRoom {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Specific(building, room) => write!(f, "{building} {room}"),
            _ => write!(f, "{}", self.get_str("display").unwrap_or_default()),
        }
    }
}

/// Represents the instructor(s) for a course
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Instructors(Option<Vec<String>>);

impl Deref for Instructors {
    type Target = Option<Vec<String>>;

    /// Deref to the inner expression
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for Instructors {
    type Err = ();

    fn from_str(instructors: &str) -> Result<Self, Self::Err> {
        let instructors = instructors
            .split(',')
            .map(|s| s.trim().to_string())
            .collect::<Vec<_>>();

        if instructors.is_empty() {
            Ok(Self(None))
        } else {
            Ok(Self(Some(instructors)))
        }
    }
}

impl From<&str> for Instructors {
    fn from(s: &str) -> Self {
        Self::from_str(s).unwrap()
    }
}

impl From<String> for Instructors {
    fn from(s: String) -> Self {
        Self::from_str(&s).unwrap()
    }
}

/// Represents a single meeting with campus and instructor
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Meeting {
    /// Days the meeting occurs
    pub days: Days,
    /// Time range for the meeting
    pub time: Option<TimeRange>,
    /// Building and room
    pub bldg_room: BuildingRoom,
    /// CMU Campus
    pub campus: String,
    /// Instructor(s) for this specific meeting
    pub instructors: Instructors,
}

/// Type of course component
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, EnumIter)]
#[cfg_attr(feature = "database", derive(DeriveActiveEnum))]
#[cfg_attr(feature = "database", sea_orm(rs_type = "String", db_type = "Text"))]
pub enum ComponentType {
    #[cfg_attr(feature = "database", sea_orm(string_value = "Lecture"))]
    Lecture,
    #[cfg_attr(feature = "database", sea_orm(string_value = "Section"))]
    Section,
}

impl FromStr for ComponentType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(if s.contains("Lec") {
            Self::Lecture
        } else {
            Self::Section
        })
    }
}

impl From<String> for ComponentType {
    fn from(s: String) -> Self {
        Self::from_str(&s).unwrap()
    }
}

/// Represents a lecture or section of a course
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CourseComponent {
    /// Course title (can vary by section)
    pub title: String,
    /// Whether this is a lecture or section
    pub component_type: ComponentType,
    /// Lecture/section code (e.g., "1", "A", "B")
    pub code: String,
    /// Meeting times for this component
    pub meetings: Vec<Meeting>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CourseNumber(String);

impl FromStr for CourseNumber {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Ensure the string is exactly 5 digits
        if s.len() == 5 && s.chars().all(|c| c.is_ascii_digit()) {
            Ok(Self(s.to_string()))
        } else {
            Err(())
        }
    }
}

impl From<&str> for CourseNumber {
    fn from(num: &str) -> Self {
        Self::from_str(num).unwrap()
    }
}

impl From<String> for CourseNumber {
    fn from(num: String) -> Self {
        Self::from_str(&num).unwrap()
    }
}

impl Display for CourseNumber {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.0)
    }
}

impl CourseNumber {
    // Format the 5-digit number in XX-XXX format
    pub fn as_full_string(&self) -> String {
        let num = &self.0;
        format!("{}-{}", &num[..2], &num[2..])
    }
}

/// Represents a course entry from the schedule
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CourseEntry {
    /// Course number (e.g., "15122")
    pub number: CourseNumber,
    /// Number of units
    pub units: Units,
    /// Lectures and sections for this course
    pub components: Vec<CourseComponent>,
    /// Season that the course is offered
    pub season: Season,
    /// Year that the course is offered
    pub year: Year,
}

/// Represents additional metadata for a course
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CourseMetadata {
    /// Related URLs for the course
    pub related_urls: Vec<String>,
    /// Whether special permission is required to take the course
    pub special_permission: bool,
    /// Description of the course
    pub description: Option<String>,
    /// The course's prerequisites
    pub prerequisites: Prerequisites,
    /// The course's corequisites
    pub corequisites: Requisites,
    /// The course's cross-listed courses
    pub crosslisted: Requisites,
    /// Notes for the course
    pub notes: Option<String>,
    /// The course's reservations
    pub reservations: Vec<Reservation>,
}

/// Represents a course object with additional metadata
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CourseObject {
    /// The base course entry
    pub course: CourseEntry,
    /// Additional metadata for the course
    pub metadata: Option<CourseMetadata>,
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::{NaiveTime, Timelike};

    #[test]
    fn test_timerange_new() {
        // Test valid time range
        let morning = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
        let noon = NaiveTime::from_hms_opt(12, 0, 0).unwrap();
        let time_range = TimeRange::new(morning, noon);
        assert!(time_range.is_some());

        // Test invalid time range (end before begin)
        let time_range = TimeRange::new(noon, morning);
        assert!(time_range.is_none());
    }

    #[test]
    fn test_timerange_from_strings() {
        // Test valid time strings
        let time_range = TimeRange::from_strings("09:30AM", "10:50AM");
        assert!(time_range.is_some());
        if let Some(range) = time_range {
            assert_eq!(range.begin.hour(), 9);
            assert_eq!(range.begin.minute(), 30);
            assert_eq!(range.end.hour(), 10);
            assert_eq!(range.end.minute(), 50);
        }

        // Test invalid time strings
        assert!(TimeRange::from_strings("not a time", "10:50AM").is_none());
        assert!(TimeRange::from_strings("09:30AM", "not a time").is_none());

        // Test invalid range (end before begin)
        assert!(TimeRange::from_strings("11:00AM", "09:00AM").is_none());
    }

    #[test]
    fn test_buildingroom_from_str() {
        // Test special cases
        assert!(matches!(
            BuildingRoom::from_str("TBA").unwrap(),
            BuildingRoom::ToBeAnnounced
        ));
        assert!(matches!(
            BuildingRoom::from_str("TBD TBD").unwrap(),
            BuildingRoom::ToBeDetermined
        ));
        assert!(matches!(
            BuildingRoom::from_str("DNM DNM").unwrap(),
            BuildingRoom::DoesNotMeet
        ));
        assert!(matches!(
            BuildingRoom::from_str("OFF PITT").unwrap(),
            BuildingRoom::OffPitt
        ));
        assert!(matches!(
            BuildingRoom::from_str("CMU REMOTE").unwrap(),
            BuildingRoom::Remote
        ));

        // Test specific building and room
        if let BuildingRoom::Specific(building, room) = BuildingRoom::from_str("GHC 5222").unwrap()
        {
            assert_eq!(building, "GHC");
            assert_eq!(room, "5222");
        } else {
            panic!("Expected BuildingRoom::Specific variant");
        }

        // Test building only
        if let BuildingRoom::Specific(building, room) = BuildingRoom::from_str("GHC").unwrap() {
            assert_eq!(building, "GHC");
            assert_eq!(room, "");
        } else {
            panic!("Expected BuildingRoom::Specific variant");
        }

        // Test multipart room
        if let BuildingRoom::Specific(building, room) =
            BuildingRoom::from_str("CUC AR 254").unwrap()
        {
            assert_eq!(building, "CUC");
            assert_eq!(room, "AR 254");
        } else {
            panic!("Expected BuildingRoom::Specific variant");
        }
    }

    #[test]
    fn test_buildingroom_display() {
        assert_eq!(BuildingRoom::ToBeAnnounced.to_string(), "TBA");
        assert_eq!(BuildingRoom::ToBeDetermined.to_string(), "TBD");
        assert_eq!(BuildingRoom::DoesNotMeet.to_string(), "DNM");
        assert_eq!(BuildingRoom::OffPitt.to_string(), "OFF PITT");
        assert_eq!(BuildingRoom::Remote.to_string(), "REMOTE");
        assert_eq!(
            BuildingRoom::Specific("GHC".to_string(), "4102".to_string()).to_string(),
            "GHC 4102"
        );
    }

    #[test]
    fn test_buildingroom_round_trip() {
        for building_room in vec![
            BuildingRoom::ToBeAnnounced,
            BuildingRoom::ToBeDetermined,
            BuildingRoom::DoesNotMeet,
            BuildingRoom::OffPitt,
            BuildingRoom::Remote,
            BuildingRoom::Specific("GHC".to_string(), "4102".to_string()),
        ] {
            let s = building_room.to_string();
            assert_eq!(BuildingRoom::from_str(&s).unwrap(), building_room);
        }
    }

    #[test]
    fn test_course_number_round_trip() {
        for course_number in [
            CourseNumber::from_str("21628").unwrap(),
            CourseNumber::from_str("21355").unwrap(),
            CourseNumber::from_str("21373").unwrap(),
        ] {
            let s = course_number.to_string();
            assert_eq!(CourseNumber::from_str(&s).unwrap(), course_number);
        }
    }
}
