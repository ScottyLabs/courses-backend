use serde::Serialize;
use std::{
    cmp::Ordering,
    fmt::{Display, Formatter, Result as FmtResult},
    str::FromStr,
};

/// Custom error type for parsing units
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum ParseUnitError {
    EmptyInput,
    NoValidUnits,
}

impl Display for ParseUnitError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::EmptyInput => write!(f, "Empty input string"),
            Self::NoValidUnits => write!(f, "No valid units found in input"),
        }
    }
}

/// Shared logic for `UnitType` and `UnitTypeSimple` to compare units
fn compare_units(min_a: f32, max_a: f32, min_b: f32, max_b: f32) -> Option<Ordering> {
    // Order first by minimum value, then by maximum value if min is equal
    match min_a.partial_cmp(&min_b) {
        Some(Ordering::Equal) => max_a.partial_cmp(&max_b),
        other => other,
    }
}

/// Represents the number of units a course is worth
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum UnitTypeSimple {
    /// A fixed number of units
    Single(f32),
    /// A range of units
    Range(f32, f32),
}

impl UnitTypeSimple {
    /// Helper to get the minimum value
    pub fn min_value(&self) -> f32 {
        match self {
            Self::Single(value) => *value,
            Self::Range(min, _) => *min,
        }
    }

    /// Helper to get the maximum value
    pub fn max_value(&self) -> f32 {
        match self {
            Self::Single(value) => *value,
            Self::Range(_, max) => *max,
        }
    }
}

impl PartialOrd for UnitTypeSimple {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        compare_units(self.min_value(), self.max_value(), other.min_value(), other.max_value())
    }
}

impl Display for UnitTypeSimple {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Single(value) => {
                // Format as whole number if it's an integer
                if value.fract() == 0.0 {
                    write!(f, "{}", *value as i32)
                } else {
                    write!(f, "{value}")
                }
            }
            Self::Range(min, max) => {
                // Format as whole numbers if they're integers
                if min.fract() == 0.0 && max.fract() == 0.0 {
                    write!(f, "{}-{}", *min as i32, *max as i32)
                } else {
                    write!(f, "{min}-{max}")
                }
            }
        }
    }
}

/// Represents the number of units a course is worth
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum UnitType {
    /// A fixed number of units
    Single(f32),
    /// A range of units
    Range(f32, f32),
    /// A combination of unit values
    Multi(Vec<UnitTypeSimple>),
}

impl UnitType {
    /// Helper to get the minimum value
    pub fn min_value(&self) -> f32 {
        match self {
            Self::Single(value) => *value,
            Self::Range(min, _) => *min,
            Self::Multi(units) => units
                .iter()
                .map(|unit| unit.min_value())
                .min_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
                .unwrap_or(0.0),
        }
    }

    /// Helper to get the maximum value
    pub fn max_value(&self) -> f32 {
        match self {
            Self::Single(value) => *value,
            Self::Range(_, max) => *max,
            Self::Multi(units) => units
                .iter()
                .map(|unit| unit.max_value())
                .max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
                .unwrap_or(0.0),
        }
    }
}

impl PartialOrd for UnitType {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        compare_units(self.min_value(), self.max_value(), other.min_value(), other.max_value())
    }
}

impl FromStr for UnitType {
    type Err = ParseUnitError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(ParseUnitError::EmptyInput);
        }

        // Check if it's a single value
        if let Ok(value) = s.parse::<f32>() {
            return Ok(Self::Single(value));
        }

        // Check if it's a single range (e.g., "3-9")
        if s.contains('-') && !s.contains(',') {
            let parts = s.split('-').collect::<Vec<_>>();
            if parts.len() == 2
                && let (Ok(min), Ok(max)) = (parts[0].parse::<f32>(), parts[1].parse::<f32>())
            {
                return Ok(Self::Range(min, max));
            }
        }

        // Handle the complex cases with commas or multiple ranges
        let mut simple_units = Vec::new();

        // First try splitting by commas
        let comma_parts: Vec<&str> = s.split(',').collect();

        for part in comma_parts {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            if part.contains('-') {
                // This is a range
                let range_parts: Vec<&str> = part.split('-').collect();
                if range_parts.len() == 2
                    && let (Ok(min), Ok(max)) =
                        (range_parts[0].parse::<f32>(), range_parts[1].parse::<f32>())
                {
                    simple_units.push(UnitTypeSimple::Range(min, max));
                    continue;
                }
            }

            // Check if it's a single value
            if let Ok(value) = part.parse::<f32>() {
                simple_units.push(UnitTypeSimple::Single(value));
                continue;
            }

            // If we get here, try to handle space-separated values
            let space_parts: Vec<&str> = part.split_whitespace().collect();
            for space_part in space_parts {
                if space_part.is_empty() {
                    continue;
                }

                if space_part.contains('-') {
                    // This is a range
                    let range_parts: Vec<&str> = space_part.split('-').collect();
                    if range_parts.len() == 2
                        && let (Ok(min), Ok(max)) =
                            (range_parts[0].parse::<f32>(), range_parts[1].parse::<f32>())
                    {
                        simple_units.push(UnitTypeSimple::Range(min, max));
                    }
                } else if let Ok(value) = space_part.parse::<f32>() {
                    // This is a single value
                    simple_units.push(UnitTypeSimple::Single(value));
                }
            }
        }

        if simple_units.is_empty() {
            return Err(ParseUnitError::NoValidUnits);
        }

        // We only have one simple unit, return it directly
        if simple_units.len() == 1 {
            match &simple_units[0] {
                UnitTypeSimple::Single(value) => return Ok(Self::Single(*value)),
                UnitTypeSimple::Range(min, max) => return Ok(Self::Range(*min, *max)),
            }
        }

        // Sort the units for consistent ordering
        simple_units.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));

        Ok(Self::Multi(simple_units))
    }
}

impl From<UnitTypeSimple> for UnitType {
    fn from(simple: UnitTypeSimple) -> Self {
        match simple {
            UnitTypeSimple::Single(value) => Self::Single(value),
            UnitTypeSimple::Range(min, max) => Self::Range(min, max),
        }
    }
}

impl From<String> for UnitType {
    fn from(s: String) -> Self {
        Self::from_str(&s).unwrap()
    }
}

impl Display for UnitType {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Single(value) => {
                // Format as whole number if it's an integer
                if value.fract() == 0.0 {
                    write!(f, "{}", *value as i32)
                } else {
                    write!(f, "{value}")
                }
            }
            Self::Range(min, max) => {
                // Format as whole numbers if they're integers
                if min.fract() == 0.0 && max.fract() == 0.0 {
                    write!(f, "{}-{}", *min as i32, *max as i32)
                } else {
                    write!(f, "{min}-{max}")
                }
            }
            Self::Multi(units) => {
                // Join all units with commas
                let mut first = true;
                for unit in units {
                    if !first {
                        write!(f, ",")?;
                    }
                    write!(f, "{unit}")?;
                    first = false;
                }
                Ok(())
            }
        }
    }
}

/// Represents how many units a course is worth
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum Units {
    /// Variable units
    VAR,
    /// A specified number of units
    Value(UnitType),
}

impl Units {
    pub fn new(value: f32) -> Self {
        Self::Value(UnitType::Single(value))
    }
}

impl PartialOrd for Units {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // VAR acts as positive infinity
        match (self, other) {
            (Self::VAR, Self::VAR) => Some(Ordering::Equal),
            (Self::VAR, Self::Value(_)) => Some(Ordering::Greater),
            (Self::Value(_), Self::VAR) => Some(Ordering::Less),
            (Self::Value(a), Self::Value(b)) => a.partial_cmp(b),
        }
    }
}

impl FromStr for Units {
    type Err = ParseUnitError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        if s.is_empty() {
            return Err(ParseUnitError::EmptyInput);
        }

        match s {
            "VAR" => Ok(Self::VAR),
            _ => UnitType::from_str(s).map(Self::Value),
        }
    }
}

impl From<UnitType> for Units {
    fn from(unit_type: UnitType) -> Self {
        Self::Value(unit_type)
    }
}

impl From<String> for Units {
    fn from(s: String) -> Self {
        Self::from_str(&s).unwrap()
    }
}

impl Display for Units {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::VAR => write!(f, "VAR"),
            Self::Value(unit_type) => write!(f, "{unit_type}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_parse_unit_type(input: &str, expected: UnitType) {
        let result = UnitType::from_str(input);
        assert!(
            result.is_ok(),
            "Failed to parse '{}': {:?}",
            input,
            result.err()
        );
        assert_eq!(result.unwrap(), expected);
    }

    fn test_parse_units(input: &str, expected: Units) {
        let result = Units::from_str(input);
        assert!(
            result.is_ok(),
            "Failed to parse '{}': {:?}",
            input,
            result.err()
        );
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_parse_single_values() {
        test_parse_unit_type("3.0", UnitType::Single(3.0));
        test_parse_unit_type("4.5", UnitType::Single(4.5));
    }

    #[test]
    fn test_parse_ranges() {
        test_parse_unit_type("3-9", UnitType::Range(3.0, 9.0));
        test_parse_unit_type("0-99", UnitType::Range(0.0, 99.0));
        test_parse_unit_type("1-48", UnitType::Range(1.0, 48.0));
    }

    #[test]
    fn test_parse_comma_separated() {
        test_parse_unit_type(
            "0,12,24",
            UnitType::Multi(vec![
                UnitTypeSimple::Single(0.0),
                UnitTypeSimple::Single(12.0),
                UnitTypeSimple::Single(24.0),
            ]),
        );

        test_parse_unit_type(
            "36,48",
            UnitType::Multi(vec![
                UnitTypeSimple::Single(36.0),
                UnitTypeSimple::Single(48.0),
            ]),
        );
    }

    #[test]
    fn test_parse_complex_combinations() {
        test_parse_unit_type(
            "3-12,18",
            UnitType::Multi(vec![
                UnitTypeSimple::Range(3.0, 12.0),
                UnitTypeSimple::Single(18.0),
            ]),
        );

        test_parse_unit_type(
            "0,36-48",
            UnitType::Multi(vec![
                UnitTypeSimple::Single(0.0),
                UnitTypeSimple::Range(36.0, 48.0),
            ]),
        );

        test_parse_unit_type(
            "3,6,12,24",
            UnitType::Multi(vec![
                UnitTypeSimple::Single(3.0),
                UnitTypeSimple::Single(6.0),
                UnitTypeSimple::Single(12.0),
                UnitTypeSimple::Single(24.0),
            ]),
        );
    }

    #[test]
    fn test_parse_special_cases() {
        test_parse_unit_type(
            "9-48 4",
            UnitType::Multi(vec![
                UnitTypeSimple::Single(4.0),
                UnitTypeSimple::Range(9.0, 48.0),
            ]),
        );
    }

    #[test]
    fn test_parse_var_units() {
        test_parse_units("VAR", Units::VAR);
    }

    #[test]
    fn test_parse_errors() {
        assert!(UnitType::from_str("").is_err());
        assert!(UnitType::from_str("not-a-number").is_err());
        assert!(UnitType::from_str("-").is_err());

        assert!(Units::from_str("").is_err());
    }

    #[test]
    fn test_ordering() {
        // Test simple ordering
        assert!(UnitType::Single(3.0) < UnitType::Single(6.0));
        assert!(UnitType::Range(1.0, 10.0) < UnitType::Range(2.0, 5.0));

        // Test range ordering based on min value
        assert!(UnitType::Range(1.0, 10.0) < UnitType::Range(2.0, 5.0));

        // Test same min, different max
        assert!(UnitType::Range(1.0, 5.0) < UnitType::Range(1.0, 10.0));

        // Test multi ordering
        let multi1 = UnitType::Multi(vec![
            UnitTypeSimple::Single(1.0),
            UnitTypeSimple::Single(3.0),
        ]);

        let multi2 = UnitType::Multi(vec![
            UnitTypeSimple::Single(2.0),
            UnitTypeSimple::Single(4.0),
        ]);

        assert!(multi1 < multi2);

        // Test Units ordering
        assert!(Units::Value(UnitType::Single(10.0)) < Units::VAR);
        assert!(Units::Value(UnitType::Single(3.0)) < Units::Value(UnitType::Single(6.0)));
    }

    #[test]
    fn test_unit_display() {
        // Test single values
        assert_eq!(UnitType::Single(3.0).to_string(), "3");
        assert_eq!(UnitType::Single(4.5).to_string(), "4.5");

        // Test ranges
        assert_eq!(UnitType::Range(3.0, 9.0).to_string(), "3-9");

        // Test multi
        let multi = UnitType::Multi(vec![
            UnitTypeSimple::Single(3.0),
            UnitTypeSimple::Range(6.0, 12.0),
        ]);
        assert_eq!(multi.to_string(), "3,6-12");

        // Test Units
        assert_eq!(Units::VAR.to_string(), "VAR");
        assert_eq!(Units::Value(UnitType::Single(9.0)).to_string(), "9");
    }
}
