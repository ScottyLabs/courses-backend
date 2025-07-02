use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    str::FromStr,
};

use serde::Serialize;
use strum::{Display, EnumString};

/// Represents the schools, departments, majors that course reservations target
#[derive(Serialize, Debug, Display, Clone, Copy, PartialEq, EnumString)]
pub enum ReservationDepartment {
    MCS,
    SCS,
    CIT,
    ISP,
    DC,
    TSB,
    ART,
    ARC,
    DRA,
    DES,
    BHA,
    ECE,
    INFOSYS,
    MSC,
}

/// Represents different types of students that reservations target
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum StudentType {
    Freshmen,
    Sophomores,
    Juniors,
    Seniors,
    Students,
}

impl FromStr for StudentType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "freshmen" => Ok(Self::Freshmen),
            "sophomores" => Ok(Self::Sophomores),
            "juniors" => Ok(Self::Juniors),
            "seniors" => Ok(Self::Seniors),
            "students" => Ok(Self::Students),
            _ => Err(format!("Unknown student type: {s}")),
        }
    }
}

impl Display for StudentType {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Freshmen => write!(f, "Freshmen"),
            Self::Sophomores => write!(f, "Sophomores"),
            Self::Juniors => write!(f, "Juniors"),
            Self::Seniors => write!(f, "Seniors"),
            Self::Students => write!(f, "Students"),
        }
    }
}

/// Represents the type of reservation restriction
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum ReservationType {
    /// Reservation for students in a specific school
    School(ReservationDepartment),
    /// Reservation for students with a primary major in a specific major
    PrimaryMajor(ReservationDepartment),
}

impl Display for ReservationType {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::School(school) => write!(f, "in {school}"),
            Self::PrimaryMajor(major) => write!(f, "with a primary major in {major}"),
        }
    }
}

/// Represents a course reservation restriction
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Restriction {
    pub student_type: StudentType,
    pub restriction_type: ReservationType,
}

impl Display for Restriction {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{} {}", self.student_type, self.restriction_type)
    }
}

impl FromStr for Restriction {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        const PREFIX: &str = "Some reservations are for ";

        // Remove the common prefix
        let content = s
            .strip_prefix(PREFIX)
            .ok_or_else(|| format!("Missing expected prefix: {PREFIX}"))?
            .trim();

        // Try primary major pattern
        if let Some((student_type_str, major_str)) = parse_primary_major_pattern(content) {
            let student_type = StudentType::from_str(student_type_str)?;
            let major = ReservationDepartment::from_str(major_str)
                .map_err(|_| format!("Unknown major: {major_str}"))?;

            return Ok(Restriction {
                student_type,
                restriction_type: ReservationType::PrimaryMajor(major),
            });
        }

        // Try school pattern
        if let Some((student_type_str, school_str)) = parse_school_pattern(content) {
            let student_type = StudentType::from_str(student_type_str)?;

            // First, try to parse as a known School
            if let Ok(school) = ReservationDepartment::from_str(school_str) {
                return Ok(Restriction {
                    student_type,
                    restriction_type: ReservationType::School(school),
                });
            }

            return Err(format!("Unknown school: {school_str}"));
        }

        Err(format!("Unable to parse reservation: {s}"))
    }
}

/// Parse pattern "Students with a primary major in INFOSYS"
fn parse_primary_major_pattern(content: &str) -> Option<(&str, &str)> {
    const PATTERN: &str = " with a primary major in ";

    content
        .split_once(PATTERN)
        .map(|(student_type, major)| (student_type.trim(), major.trim()))
}

/// Parse pattern "Freshmen in SCS" or "Students in ECE"
fn parse_school_pattern(content: &str) -> Option<(&str, &str)> {
    const PATTERN: &str = " in ";

    content
        .split_once(PATTERN)
        .map(|(student_type, school)| (student_type.trim(), school.trim()))
}

/// Represents a course reservation for a section
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Reservation {
    pub section: String,
    pub restrictions: Vec<Restriction>,
}

#[cfg(test)]
mod tests {
    use crate::reservation::{ReservationDepartment, ReservationType, Restriction, StudentType};
    use std::str::FromStr;

    #[test]
    fn test_student_type_parsing() {
        assert_eq!(
            StudentType::from_str("Freshmen").unwrap(),
            StudentType::Freshmen
        );
        assert_eq!(
            StudentType::from_str("Students").unwrap(),
            StudentType::Students
        );
        assert_eq!(
            StudentType::from_str("Seniors").unwrap(),
            StudentType::Seniors
        );
    }

    #[test]
    fn test_restriction_department_parsing() {
        let restriction =
            Restriction::from_str("Some reservations are for Freshmen in SCS").unwrap();
        assert_eq!(restriction.student_type, StudentType::Freshmen);
        assert_eq!(
            restriction.restriction_type,
            ReservationType::School(ReservationDepartment::SCS)
        );

        let restriction =
            Restriction::from_str("Some reservations are for Students in ECE").unwrap();
        assert_eq!(restriction.student_type, StudentType::Students);
        assert_eq!(
            restriction.restriction_type,
            ReservationType::School(ReservationDepartment::ECE)
        );
    }

    #[test]
    fn test_restriction_primary_major_parsing() {
        let restriction = Restriction::from_str(
            "Some reservations are for Students with a primary major in INFOSYS",
        )
        .unwrap();
        assert_eq!(restriction.student_type, StudentType::Students);
        assert_eq!(
            restriction.restriction_type,
            ReservationType::PrimaryMajor(ReservationDepartment::INFOSYS)
        );

        let restriction =
            Restriction::from_str("Some reservations are for Freshmen with a primary major in BHA")
                .unwrap();
        assert_eq!(restriction.student_type, StudentType::Freshmen);
        assert_eq!(
            restriction.restriction_type,
            ReservationType::PrimaryMajor(ReservationDepartment::BHA)
        );
    }

    #[test]
    fn test_restriction_display() {
        let restriction = Restriction {
            student_type: StudentType::Freshmen,
            restriction_type: ReservationType::School(ReservationDepartment::SCS),
        };
        assert_eq!(restriction.to_string(), "Freshmen in SCS");

        let restriction = Restriction {
            student_type: StudentType::Students,
            restriction_type: ReservationType::PrimaryMajor(ReservationDepartment::INFOSYS),
        };
        assert_eq!(
            restriction.to_string(),
            "Students with a primary major in INFOSYS"
        );
    }
}
