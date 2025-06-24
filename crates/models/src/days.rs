use serde::Serialize;
use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not},
    str::FromStr,
};

/// Represents the days of the week a meeting occurs
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[repr(transparent)]
pub struct DaySet(u8);

impl DaySet {
    // Constants for individual days
    pub const MONDAY: Self = DaySet(1 << 0);
    pub const TUESDAY: Self = DaySet(1 << 1);
    pub const WEDNESDAY: Self = DaySet(1 << 2);
    pub const THURSDAY: Self = DaySet(1 << 3);
    pub const FRIDAY: Self = DaySet(1 << 4);
    pub const SATURDAY: Self = DaySet(1 << 5);
    pub const SUNDAY: Self = DaySet(1 << 6);

    // Constants for common day combinations
    pub const WEEKDAYS: Self = DaySet(0b0011111);
    pub const WEEKEND: Self = DaySet(0b1100000);
    pub const ALL: Self = DaySet(0b1111111);
    pub const NONE: Self = DaySet(0);

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

        write!(f, "{result}")
    }
}

// Bitwise operators
impl BitOr for DaySet {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        DaySet(self.0 | rhs.0)
    }
}

impl BitAnd for DaySet {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        DaySet(self.0 & rhs.0)
    }
}

impl Not for DaySet {
    type Output = Self;

    fn not(self) -> Self::Output {
        // Apply mask to keep only 7 bits
        DaySet((!self.0) & 0x7F)
    }
}

impl BitOrAssign for DaySet {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAndAssign for DaySet {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

/// Represents when a meeting can occur
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize)]
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

#[cfg(test)]
mod test {
    use crate::days::DaySet;
    use std::str::FromStr;

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
}
