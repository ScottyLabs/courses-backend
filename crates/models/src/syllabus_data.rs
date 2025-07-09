use chrono::{Datelike, Utc};
use serde::Serialize;
use std::{
    collections::HashMap,
    fmt::{Display as FmtDisplay, Formatter, Result as FmtResult},
    hash::Hash,
    num::ParseIntError,
    ops::Deref,
    str::FromStr,
};
use strum::{
    AsRefStr, Display, EnumIter, EnumProperty, EnumString, IntoEnumIterator, IntoStaticStr,
};

/// Type alias for a map that associates course metadata with file URLs
pub type SyllabusMap = HashMap<(Year, Season, String, String), String>;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, EnumString, EnumIter, AsRefStr, EnumProperty,
)]
pub enum Season {
    #[strum(serialize = "F", props(full = "fall"))]
    Fall,
    #[strum(serialize = "S", props(full = "spring"))]
    Spring,
    #[strum(serialize = "M", props(full = "summer_1"))]
    Summer1,
    #[strum(serialize = "N", props(full = "summer_2"))]
    Summer2,
}

impl Season {
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }

    pub fn as_full_str(&self) -> &'static str {
        self.get_str("full").unwrap_or_default()
    }

    pub fn all() -> Vec<Season> {
        Season::iter().collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct Year(pub u16);

impl Year {
    pub fn all() -> Vec<Year> {
        let current_year = Utc::now().year() as u16;
        (2018..=current_year).map(Year).collect()
    }
}

impl Deref for Year {
    type Target = u16;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for Year {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let year = s.parse()?;
        Ok(Year(year))
    }
}

impl FmtDisplay for Year {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{:02}", self.0 % 100)
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Serialize, Display, EnumString, EnumIter, IntoStaticStr,
)]
pub enum Department {
    CB,
    BSC,
    ICT,
    HCI,
    CHE,
    SCS,
    CMY,
    MLG,
    LTI,
    CEE,
    INI,
    CS,
    ROB,
    S3D,
    ECE,
    EPP,
    MSC,
    MEG,
    MSE,
    NVS,
    PHY,
    STA,
    MCS,
    CIT,
    BMD,
    BUS,
    ARC,
    III,
    DES,
    BXA,
    ETC,
    DRA,
    MUS,
    ART,
    BSA,
    CAS,
    H00,
    HSS,
    ISP,
    PE,
    ECO,
    ENG,
    HIS,
    PHI,
    ML,
    CST,
    PSY,
    CNB,
    SDS,
    PPP,
    PMP,
    MED,
    AEM,
    HC,
    ISM,
    STU,
    CMU,
}

impl Department {
    pub fn all() -> Vec<Department> {
        Department::iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_department_display() {
        assert_eq!(Department::CS.to_string(), "CS");
        assert_eq!(Department::ROB.to_string(), "ROB");
        assert_eq!(Department::MSC.to_string(), "MSC");
    }

    #[test]
    fn test_department_from_str() {
        assert_eq!(Department::from_str("CS").unwrap(), Department::CS);
        assert_eq!(Department::from_str("ROB").unwrap(), Department::ROB);
        assert_eq!(Department::from_str("MSC").unwrap(), Department::MSC);
    }

    #[test]
    fn test_department_all() {
        let all = Department::all();
        assert_eq!(all.len(), 57);
    }

    #[test]
    fn test_season_as_str() {
        assert_eq!(Season::Fall.as_str(), "F");
        assert_eq!(Season::Summer2.as_str(), "N");
    }

    #[test]
    fn test_season_as_full_str() {
        assert_eq!(Season::Spring.as_full_str(), "spring");
        assert_eq!(Season::Summer1.as_full_str(), "summer_1");
    }

    #[test]
    fn test_season_from_str() {
        assert_eq!(Season::from_str("F").unwrap(), Season::Fall);
        assert_eq!(Season::from_str("N").unwrap(), Season::Summer2);
    }

    #[test]
    fn test_season_all() {
        let all = Season::all();
        assert_eq!(all.len(), 4);
    }

    #[test]
    fn test_season_round_trip() {
        for season in Season::all() {
            let s = season.as_str();
            assert_eq!(Season::from_str(s).unwrap(), season);
        }
    }

    #[test]
    fn test_year_display() {
        assert_eq!(Year(2020).to_string(), "20");
        assert_eq!(Year(2023).to_string(), "23");
    }

    #[test]
    fn test_year_all() {
        let all = Year::all();
        let current_year = Utc::now().year() as u16;

        assert!(all.contains(&Year(2018)));
        assert!(all.contains(&Year(current_year)));
    }
}
