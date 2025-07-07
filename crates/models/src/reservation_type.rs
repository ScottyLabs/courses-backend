use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result as FmtResult};

#[cfg(feature = "database")]
use sea_orm::Value;

/// Represents the type of reservation restriction
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ReservationType {
    /// Reservation for a specific type of student
    StudentType,
    /// Reservation for students in a specific school
    School(String),
    /// Reservation for students with a primary major in a specific major
    PrimaryMajor(String),
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

#[cfg(feature = "database")]
impl sea_orm::sea_query::ValueType for ReservationType {
    fn try_from(v: Value) -> Result<Self, sea_orm::sea_query::ValueTypeErr> {
        match v {
            Value::String(Some(s)) => {
                // Try to deserialize from JSON string
                serde_json::from_str(&s).map_err(|_| sea_orm::sea_query::ValueTypeErr)
            }
            _ => Err(sea_orm::sea_query::ValueTypeErr),
        }
    }

    fn type_name() -> String {
        "ReservationType".to_string()
    }

    fn array_type() -> sea_orm::sea_query::ArrayType {
        sea_orm::sea_query::ArrayType::String
    }

    fn column_type() -> sea_orm::sea_query::ColumnType {
        sea_orm::sea_query::ColumnType::Text
    }
}

#[cfg(feature = "database")]
impl From<ReservationType> for Value {
    fn from(reservation_type: ReservationType) -> Self {
        // Serialize to JSON string for database storage
        Value::String(Some(Box::new(
            serde_json::to_string(&reservation_type).unwrap(),
        )))
    }
}

#[cfg(feature = "database")]
impl sea_orm::TryGetable for ReservationType {
    fn try_get_by<I: sea_orm::ColIdx>(
        res: &sea_orm::QueryResult,
        index: I,
    ) -> Result<Self, sea_orm::TryGetError> {
        let val: String = res.try_get_by(index)?;

        serde_json::from_str(&val).map_err(|e| {
            sea_orm::TryGetError::DbErr(sea_orm::DbErr::Type(format!(
                "Failed to deserialize ReservationType: {e}"
            )))
        })
    }
}

#[cfg(feature = "database")]
impl sea_orm::sea_query::Nullable for ReservationType {
    fn null() -> Value {
        Value::String(None)
    }
}
