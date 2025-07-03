use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    str::FromStr,
};

use serde::Serialize;
use strum::{Display, EnumString};

/// Represents the schools, departments, majors that course reservations target
#[derive(Serialize, Debug, Display, Clone, Copy, PartialEq, EnumString)]
pub enum ReservationDepartment {
    ADS,
    AEM,
    AI,
    AIEBMD,
    AIEESTAP,
    AIEINI,
    AIEMEG,
    AII,
    ALSLA,
    ARC,
    ARCCM,
    ART,
    ARTMGMT,
    ASLA,
    AUTSCBE,
    BA,
    BEHAVECO,
    BHA,
    BIDA,
    BMD,
    BPD,
    BSC,
    BSCCBIO,
    BTPE,
    BUS,
    BUSANAL,
    BXA,
    C00,
    CAS,
    CB,
    CEE,
    CFA,
    CHE,
    CIT,
    CIV,
    CMU,
    CMY,
    CNB,
    COGSCI,
    COLPIA,
    COMP,
    COMPFIN,
    COSDES,
    COSPRO,
    CRM,
    CS,
    CST,
    CV,
    CW,
    DATANSCI,
    DC,
    DECSCI,
    DES,
    DIR,
    DRA,
    DRAWRT,
    ECE,
    ECO,
    EHPP,
    ELECMUS,
    EMSYS,
    ENARTINT,
    ENG,
    ENTMGMT,
    ENVENG,
    EPP,
    ESDES,
    ETC,
    ETIM,
    FILVIM,
    GCP,
    H00,
    HC,
    HCAIT,
    HCI,
    HCP,
    HIS,
    HSS,
    IA,
    ICT,
    III,
    IIPS,
    INAP,
    INFOPVCY,
    INFOSYS,
    INI,
    INTRELP,
    IPM,
    IPS,
    ISH,
    ISM,
    ISP,
    ISR,
    ITM,
    LCL,
    LITCUL,
    LNGHSS,
    LTI,
    MCS,
    MED,
    MEG,
    MEGRE,
    MGMT,
    MGTSCI,
    MIS,
    ML,
    MLG,
    MPBP,
    MPPIA,
    MPVOI,
    MSC,
    MSCORSTA,
    MSE,
    MUS,
    MUSCOM,
    MUSCOMP,
    MUSTEC,
    NEUROSCI,
    NSI,
    PHI,
    PHY,
    PMP,
    POLMGMT,
    PPIA,
    PPMDA,
    PPMHNZ,
    PPP,
    PRE,
    PRODMGMT,
    PROJECT,
    PSY,
    PVOI,
    PW,
    QBA,
    QBIOBAD,
    QBS,
    QCS,
    QIS,
    ROB,
    ROBSYSDV,
    S3D,
    SCASYS,
    SCS,
    SDS,
    SE,
    SM,
    SOUNDDES,
    STA,
    STAMACH,
    STU,
    SUDESAD,
    SV,
    TA,
    TEX,
    TSB,
    TW,
    Z00,
}

/// Represents different types of students that reservations target
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum StudentType {
    Freshmen,
    Sophomores,
    Juniors,
    Seniors,
    Students,
    GraduateStudents,
    PhdCandidates,
    FifthYearStudents,
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
            "graduate students" => Ok(Self::GraduateStudents),
            "phd candidates" => Ok(Self::PhdCandidates),
            "5th yr students" => Ok(Self::FifthYearStudents),
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
            Self::GraduateStudents => write!(f, "Graduate Students"),
            Self::PhdCandidates => write!(f, "Phd Candidates"),
            Self::FifthYearStudents => write!(f, "5th YR Students"),
        }
    }
}

/// Represents the type of reservation restriction
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum ReservationType {
    /// Reservation for a specific type of student
    StudentType,
    /// Reservation for students in a specific school
    School(ReservationDepartment),
    /// Reservation for students with a primary major in a specific major
    PrimaryMajor(ReservationDepartment),
}

impl Display for ReservationType {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::StudentType => Ok(()),
            Self::School(school) => write!(f, "in {school}"),
            Self::PrimaryMajor(major) => write!(f, "with a primary major in {major}"),
        }
    }
}

/// Represents a course reservation restriction
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Restriction {
    pub student_type: Option<StudentType>,
    pub restriction_type: Option<ReservationType>,
}

impl Display for Restriction {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match (self.student_type.clone(), self.restriction_type.clone()) {
            (Some(student_type), Some(restriction_type)) => {
                write!(f, "{student_type} {restriction_type}")
            }
            _ => write!(f, "Unknown restriction"),
        }
    }
}

impl FromStr for Restriction {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Remove the common prefix
        const PREFIX: &str = "Some reservations are for ";

        let content = s
            .strip_prefix(PREFIX)
            .ok_or("Missing expected prefix")?
            .trim();

        // Remove empty lines and whitespace
        let s = s
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join(" ");

        // Try primary major pattern
        if let Some((student_type_str, major_str)) = parse_primary_major_pattern(content) {
            let student_type = StudentType::from_str(student_type_str)?;
            let major = ReservationDepartment::from_str(major_str)
                .map_err(|_| format!("Unknown major: {major_str}"))?;

            return Ok(Restriction {
                student_type: Some(student_type),
                restriction_type: Some(ReservationType::PrimaryMajor(major)),
            });
        }

        // Try school pattern
        if let Some((student_type_str, school_str)) = parse_school_pattern(content) {
            let student_type = StudentType::from_str(student_type_str)?;

            // First, try to parse as a known School
            if let Ok(school) = ReservationDepartment::from_str(school_str) {
                return Ok(Restriction {
                    student_type: Some(student_type),
                    restriction_type: Some(ReservationType::School(school)),
                });
            }

            return Err(format!("Unknown school: {school_str}"));
        }

        // Try just student type pattern
        if let Ok(student_type) = StudentType::from_str(content) {
            return Ok(Restriction {
                student_type: Some(student_type),
                restriction_type: Some(ReservationType::StudentType),
            });
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
        assert_eq!(restriction.student_type, Some(StudentType::Freshmen));
        assert_eq!(
            restriction.restriction_type,
            Some(ReservationType::School(ReservationDepartment::SCS))
        );

        let restriction =
            Restriction::from_str("Some reservations are for Students in ECE").unwrap();
        assert_eq!(restriction.student_type, Some(StudentType::Students));
        assert_eq!(
            restriction.restriction_type,
            Some(ReservationType::School(ReservationDepartment::ECE))
        );
    }

    #[test]
    fn test_restriction_primary_major_parsing() {
        let restriction = Restriction::from_str(
            "Some reservations are for Students with a primary major in INFOSYS",
        )
        .unwrap();
        assert_eq!(restriction.student_type, Some(StudentType::Students));
        assert_eq!(
            restriction.restriction_type,
            Some(ReservationType::PrimaryMajor(
                ReservationDepartment::INFOSYS
            ))
        );

        let restriction =
            Restriction::from_str("Some reservations are for Freshmen with a primary major in BHA")
                .unwrap();
        assert_eq!(restriction.student_type, Some(StudentType::Freshmen));
        assert_eq!(
            restriction.restriction_type,
            Some(ReservationType::PrimaryMajor(ReservationDepartment::BHA))
        );
    }

    #[test]
    fn test_restriction_student_type_parsing() {
        let restriction =
            Restriction::from_str("Some reservations are for 5th YR Students").unwrap();
        assert_eq!(
            restriction.student_type,
            Some(StudentType::FifthYearStudents)
        );
        assert_eq!(
            restriction.restriction_type,
            Some(ReservationType::StudentType)
        );

        let restriction =
            Restriction::from_str("Some reservations are for Phd Candidates").unwrap();
        assert_eq!(restriction.student_type, Some(StudentType::PhdCandidates));
        assert_eq!(
            restriction.restriction_type,
            Some(ReservationType::StudentType)
        );

        let restriction =
            Restriction::from_str("Some reservations are for Graduate Students").unwrap();
        assert_eq!(
            restriction.student_type,
            Some(StudentType::GraduateStudents)
        );
        assert_eq!(
            restriction.restriction_type,
            Some(ReservationType::StudentType)
        );
    }

    #[test]
    fn test_restriction_display() {
        let restriction = Restriction {
            student_type: Some(StudentType::Freshmen),
            restriction_type: Some(ReservationType::School(ReservationDepartment::SCS)),
        };
        assert_eq!(restriction.to_string(), "Freshmen in SCS");

        let restriction = Restriction {
            student_type: Some(StudentType::Students),
            restriction_type: Some(ReservationType::PrimaryMajor(
                ReservationDepartment::INFOSYS,
            )),
        };
        assert_eq!(
            restriction.to_string(),
            "Students with a primary major in INFOSYS"
        );
    }
}
