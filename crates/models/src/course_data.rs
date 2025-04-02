use crate::{
    syllabus_data::{Season, Year},
    units::Units,
};
use chrono::NaiveTime;
use serde::Serialize;
use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not},
    str::FromStr,
};
use strum::{EnumIter, EnumProperty, IntoEnumIterator};

/// Represents the days of the week a meeting occurs
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct DaySet {
    days: u8,
}

impl DaySet {
    // Constants for individual days
    pub const MONDAY: Self = DaySet { days: 1 << 0 };
    pub const TUESDAY: Self = DaySet { days: 1 << 1 };
    pub const WEDNESDAY: Self = DaySet { days: 1 << 2 };
    pub const THURSDAY: Self = DaySet { days: 1 << 3 };
    pub const FRIDAY: Self = DaySet { days: 1 << 4 };
    pub const SATURDAY: Self = DaySet { days: 1 << 5 };
    pub const SUNDAY: Self = DaySet { days: 1 << 6 };

    // Constants for common day combinations
    pub const WEEKDAYS: Self = DaySet { days: 0b0011111 };
    pub const WEEKEND: Self = DaySet { days: 0b1100000 };
    pub const ALL: Self = DaySet { days: 0b1111111 };
    pub const NONE: Self = DaySet { days: 0 };

    /// Day-to-char mapping for parsing and display
    const DAY_CHARS: [(Self, char); 7] = [
        (Self::MONDAY, 'M'),
        (Self::TUESDAY, 'T'),
        (Self::WEDNESDAY, 'W'),
        (Self::THURSDAY, 'R'),
        (Self::FRIDAY, 'F'),
        (Self::SATURDAY, 'S'),
        (Self::SUNDAY, 'U'),
    ];

    pub fn new() -> Self {
        Self::NONE
    }

    pub fn contains(self, day: Self) -> bool {
        (self & day) == day
    }

    pub fn set(&mut self, day: Self, value: bool) {
        if value {
            *self |= day;
        } else {
            *self &= !day;
        }
    }

    pub fn add(&mut self, day: Self) {
        *self |= day;
    }

    pub fn remove(&mut self, day: Self) {
        *self &= !day;
    }
}

impl FromStr for DaySet {
    type Err = ();

    fn from_str(days: &str) -> Result<Self, Self::Err> {
        let mut result = Self::NONE;

        for c in days.chars() {
            for &(day, day_char) in &Self::DAY_CHARS {
                if c == day_char {
                    result |= day;
                    break;
                }
            }
        }

        Ok(result)
    }
}

impl Display for DaySet {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let mut result = String::new();

        for &(day, day_char) in &Self::DAY_CHARS {
            if self.contains(day) {
                result.push(day_char);
            }
        }

        write!(f, "{}", result)
    }
}

// Bitwise operators
impl BitOr for DaySet {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        DaySet {
            days: self.days | rhs.days,
        }
    }
}

impl BitAnd for DaySet {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        DaySet {
            days: self.days & rhs.days,
        }
    }
}

impl Not for DaySet {
    type Output = Self;

    fn not(self) -> Self::Output {
        // Apply mask to keep only 7 bits
        DaySet {
            days: (!self.days) & 0x7F,
        }
    }
}

impl BitOrAssign for DaySet {
    fn bitor_assign(&mut self, rhs: Self) {
        self.days |= rhs.days;
    }
}

impl BitAndAssign for DaySet {
    fn bitand_assign(&mut self, rhs: Self) {
        self.days &= rhs.days;
    }
}

/// Represents when a meeting can occur
#[derive(Debug, Clone, Copy, Default, Serialize)]
pub enum Days {
    /// Specific days
    Days(DaySet),
    #[default]
    /// To be announced
    TBA,
}

impl FromStr for Days {
    type Err = ();

    fn from_str(days: &str) -> Result<Self, Self::Err> {
        if days.contains("TBA") {
            Ok(Self::TBA)
        } else {
            DaySet::from_str(days).map(Self::Days)
        }
    }
}

impl From<String> for Days {
    fn from(days: String) -> Self {
        Self::from_str(&days).unwrap_or_default()
    }
}

/// Represents a time range for a meeting
#[derive(Debug, Clone, Copy, Serialize)]
pub struct TimeRange {
    pub begin: NaiveTime,
    pub end: NaiveTime,
}

impl TimeRange {
    /// Creates a new `TimeRange` if `begin` is before `end`
    pub fn new(begin: NaiveTime, end: NaiveTime) -> Option<Self> {
        (begin < end).then_some(Self { begin, end })
    }

    /// Parses two time strings and creates a `TimeRange` if valid.
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
#[derive(Debug, Clone, Serialize, EnumProperty, EnumIter)]
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
            .find(|v| v.get_str("parse") == Some(bldg_room))
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
            Self::Specific(building, room) => write!(f, "{} {}", building, room),
            _ => write!(f, "{}", self.get_str("display").unwrap_or_default()),
        }
    }
}

/// Represents a location where a meeting can occur
#[derive(Debug, Clone, Serialize, EnumProperty, EnumIter)]
pub enum Location {
    #[strum(props(display = "Pittsburgh, Pennsylvania"))]
    Pittsburgh,

    #[strum(props(display = "Doha, Qatar"))]
    Doha,

    #[strum(props(display = "New York, New York"))]
    NewYork,

    #[strum(props(display = "San Jose, California"))]
    SanJose,

    #[strum(props(display = "Los Angeles, California"))]
    LosAngeles,

    #[strum(props(display = "Lisbon, Portugal"))]
    Lisbon,

    #[strum(props(display = "Kigali, Rwanda"))]
    Kigali,

    #[strum(props(display = "Washington, District of Columbia"))]
    Washington,

    #[strum(props(display = "Unknown Location"))]
    Unknown,

    /// Any other location
    #[strum(props(display = ""))]
    Other(String),
}

impl FromStr for Location {
    type Err = ();

    fn from_str(location: &str) -> Result<Self, Self::Err> {
        Self::iter()
            .find(|variant| variant.get_str("display") == Some(location))
            .or_else(|| Some(Self::Other(location.to_string())))
            .ok_or(())
    }
}

impl From<String> for Location {
    fn from(location: String) -> Self {
        Self::from_str(&location).unwrap()
    }
}

impl Display for Location {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Other(location) => write!(f, "{}", location),
            _ => {
                let display = self.get_str("display").unwrap_or_default();
                write!(f, "{}", display)
            }
        }
    }
}

/// Represents a single meeting with location and instructor
#[derive(Debug, Clone, Serialize)]
pub struct Meeting {
    /// Days the meeting occurs
    pub days: Days,
    /// Time range for the meeting
    pub time: Option<TimeRange>,
    /// Building and room
    pub bldg_room: BuildingRoom,
    /// Location (campus)
    pub location: Location,
    /// Instructor(s) for this specific meeting
    pub instructors: String,
}

/// Type of course component
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum ComponentType {
    Lecture,
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
#[derive(Debug, Clone, Serialize)]
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

/// Represents a course entry from the schedule
#[derive(Debug, Clone, Serialize)]
pub struct CourseEntry {
    /// Course number (e.g., "15122")
    pub number: String,
    /// Number of units
    pub units: Units,
    /// Lectures and sections for this course
    pub components: Vec<CourseComponent>,
    /// Season that the course is offered
    pub season: Season,
    /// Year that the course is offered
    pub year: Year,
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::{NaiveTime, Timelike};

    #[test]
    fn test_day_set_from_str() {
        let days = DaySet::from_str("MWF").unwrap();
        assert!(days.contains(DaySet::MONDAY));
        assert!(!days.contains(DaySet::TUESDAY));
        assert!(days.contains(DaySet::WEDNESDAY));
        assert!(!days.contains(DaySet::THURSDAY));
        assert!(days.contains(DaySet::FRIDAY));
        assert!(!days.contains(DaySet::SATURDAY));
        assert!(!days.contains(DaySet::SUNDAY));
    }

    #[test]
    fn test_day_set_display() {
        let mut days = DaySet::new();
        days.add(DaySet::MONDAY);
        days.add(DaySet::WEDNESDAY);
        days.add(DaySet::FRIDAY);

        assert_eq!(days.to_string(), "MWF");
    }

    #[test]
    fn test_day_set_bitwise_operations() {
        let mwf = DaySet::MONDAY | DaySet::WEDNESDAY | DaySet::FRIDAY;
        assert!(mwf.contains(DaySet::MONDAY));
        assert!(!mwf.contains(DaySet::TUESDAY));
        assert!(mwf.contains(DaySet::WEDNESDAY));
        assert!(!mwf.contains(DaySet::THURSDAY));
        assert!(mwf.contains(DaySet::FRIDAY));

        let weekdays = DaySet::WEEKDAYS;
        assert_eq!(weekdays.to_string(), "MTWRF");
    }

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
    fn test_location_from_str() {
        // Test known locations
        assert!(matches!(
            Location::from_str("Pittsburgh, Pennsylvania").unwrap(),
            Location::Pittsburgh
        ));
        assert!(matches!(
            Location::from_str("Doha, Qatar").unwrap(),
            Location::Doha
        ));

        // Test other location
        if let Location::Other(loc) = Location::from_str("Adelaide, Australia").unwrap() {
            assert_eq!(loc, "Adelaide, Australia");
        } else {
            panic!("Expected Location::Other variant");
        }
    }

    #[test]
    fn test_location_display() {
        assert_eq!(Location::Pittsburgh.to_string(), "Pittsburgh, Pennsylvania");
        assert_eq!(
            Location::Other("Adelaide, Australia".to_string()).to_string(),
            "Adelaide, Australia"
        );
    }
}
