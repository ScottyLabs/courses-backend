use std::str::FromStr;

/// Represents a node in the expression tree
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Course(String),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
}

impl Expr {
    /// Evaluate if this expression is satisfied by the given completed courses
    pub fn evaluate(&self, completed_courses: &[String]) -> bool {
        match self {
            Expr::Course(course) => completed_courses.contains(course),
            Expr::And(left, right) => {
                left.evaluate(completed_courses) && right.evaluate(completed_courses)
            }
            Expr::Or(left, right) => {
                left.evaluate(completed_courses) || right.evaluate(completed_courses)
            }
        }
    }

    /// Simplifies this expression based on completed courses
    /// - Returns None if the requirement is already satisfied
    /// - Returns a simplified [`Expr`] showing only remaining requirements otherwise
    pub fn simplify(&self, completed_courses: &[String]) -> Option<Expr> {
        match self {
            // For a course node, if it's completed return None (satisfied)
            // otherwise return the course requirement
            Expr::Course(course) => {
                if completed_courses.contains(course) {
                    None // Course is already completed
                } else {
                    Some(Expr::Course(course.clone())) // Course still needed
                }
            }

            // For AND nodes, both sides must be satisfied
            Expr::And(left, right) => {
                match (
                    left.simplify(completed_courses),
                    right.simplify(completed_courses),
                ) {
                    (None, None) => None,       // Both sides satisfied
                    (Some(l), None) => Some(l), // Only right side satisfied
                    (None, Some(r)) => Some(r), // Only left side satisfied
                    (Some(l), Some(r)) => Some(Expr::And(Box::new(l), Box::new(r))), // Both sides have remaining requirements
                }
            }

            // For OR nodes, only one side needs to be satisfied
            Expr::Or(left, right) => {
                match (
                    left.simplify(completed_courses),
                    right.simplify(completed_courses),
                ) {
                    (None, _) | (_, None) => None, // Either side satisfied means OR is satisfied
                    (Some(l), Some(r)) => Some(Expr::Or(Box::new(l), Box::new(r))), // Both sides have remaining requirements
                }
            }
        }
    }
}

/// Custom error type for parsing requisites
#[derive(Debug)]
pub struct ParseError(pub String);

impl FromStr for Expr {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let input = s.trim();

        // Handle parenthesized expressions
        if input.starts_with('(') && input.ends_with(')') {
            // Find matching parentheses by counting
            let mut count = 0;
            let mut found_closing = false;

            for (i, ch) in input.chars().enumerate() {
                if ch == '(' {
                    count += 1;
                } else if ch == ')' {
                    count -= 1;
                    if count == 0 && i < input.len() - 1 {
                        // We found a closing parenthesis before the end, so this isn't a simple parenthesized expression
                        found_closing = true;
                        break;
                    }
                }
            }

            if !found_closing {
                // This is a simple parenthesized expression, extract inner content
                let inner = &input[1..input.len() - 1].trim();
                return inner.parse();
            }
        }

        // Try to parse as OR expression first (lower precedence)
        if let Some(or_parts) = split_top_level(input, "or")
            && or_parts.len() >= 2
        {
            let mut result = or_parts[0].parse()?;
            for part in &or_parts[1..] {
                result = Expr::Or(Box::new(result), Box::new(part.parse()?));
            }
            return Ok(result);
        }

        // Try to parse as AND expression (higher precedence)
        if let Some(and_parts) = split_top_level(input, "and")
            && and_parts.len() >= 2
        {
            let mut result = and_parts[0].parse()?;
            for part in &and_parts[1..] {
                result = Expr::And(Box::new(result), Box::new(part.parse()?));
            }
            return Ok(result);
        }

        // Check if it's a parenthesized expression
        if input.starts_with('(') && input.ends_with(')') {
            let inner = &input[1..input.len() - 1].trim();
            return inner.parse();
        }

        // If no operators found and input is just digits, it's a course number
        if input.chars().all(|c| c.is_ascii_digit()) {
            return Ok(Expr::Course(input.to_string()));
        }

        Err(ParseError(format!("Failed to parse: {input}")))
    }
}

impl From<String> for Expr {
    fn from(s: String) -> Self {
        s.parse().unwrap()
    }
}

/// Split a string on a top level operator, respecting nested parentheses
fn split_top_level(input: &str, op: &str) -> Option<Vec<String>> {
    let mut result = Vec::new();
    let mut paren_count = 0;
    let mut start = 0;
    let mut i = 0;
    let chars: Vec<char> = input.chars().collect();

    while i < chars.len() {
        match chars[i] {
            '(' => paren_count += 1,
            ')' => paren_count -= 1,
            ' ' if paren_count == 0 && i + op.len() + 2 <= input.len() => {
                // Need to match pattern " op " (with spaces on both sides)
                let slice = &input[i..i + op.len() + 2];
                if slice.starts_with(' ') && slice[1..].starts_with(op) && slice.ends_with(' ') {
                    let part = input[start..i].trim();
                    if !part.is_empty() {
                        result.push(part.to_string());
                    }

                    i += op.len() + 1; // Skip the operator and one space
                    start = i + 1;
                }
            }
            _ => {}
        }

        i += 1;
    }

    // Add the last part
    let last_part = input[start..].trim();
    if !last_part.is_empty() {
        result.push(last_part.to_string());
    }

    if result.len() >= 2 {
        Some(result)
    } else {
        None
    }
}

/// Parse a requirement string and return a structured expression tree
pub fn parse_requirements(req_str: &str) -> Result<Expr, ParseError> {
    req_str.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_parsing() {
        assert!(matches!("15122".parse::<Expr>().unwrap(), Expr::Course(s) if s == "15122"));

        let expr = "15122 and 21122".parse::<Expr>().unwrap();
        assert!(matches!(expr, Expr::And(_, _)));

        let expr = "15122 or 21122".parse::<Expr>().unwrap();
        assert!(matches!(expr, Expr::Or(_, _)));
    }

    #[test]
    fn test_complex_expression() {
        let expr = "(15122 and 21122) or 15213".parse::<Expr>().unwrap();

        // Test evaluation with different course sets
        assert!(expr.evaluate(&["15122".to_string(), "21122".to_string()]));
        assert!(expr.evaluate(&["15213".to_string()]));
        assert!(!expr.evaluate(&["15122".to_string()]));

        let req_str = "(15122 and 21122 and 21240) or (15122 and 21122 and 21241)";
        let expr = req_str.parse::<Expr>().unwrap();

        assert!(!expr.evaluate(&["15122".to_string(), "21122".to_string()]));
        assert!(expr.evaluate(&[
            "15122".to_string(),
            "21122".to_string(),
            "21240".to_string()
        ]));
        assert!(expr.evaluate(&[
            "15122".to_string(),
            "21122".to_string(),
            "21241".to_string()
        ]));
    }

    #[test]
    fn test_simplify_course() {
        // Test a single course
        let expr = Expr::Course("15122".to_string());

        // Not completed
        let simplified = expr.simplify(&[]);
        assert_eq!(simplified, Some(Expr::Course("15122".to_string())));

        // Completed
        let simplified = expr.simplify(&["15122".to_string()]);
        assert_eq!(simplified, None);
    }

    #[test]
    fn test_simplify_and() {
        // Test AND expression
        let expr = Expr::And(
            Box::new(Expr::Course("15122".to_string())),
            Box::new(Expr::Course("21122".to_string())),
        );

        // Nothing completed
        let simplified = expr.simplify(&[]);
        assert_eq!(simplified, Some(expr.clone()));

        // First course completed
        let simplified = expr.simplify(&["15122".to_string()]);
        assert_eq!(simplified, Some(Expr::Course("21122".to_string())));

        // Second course completed
        let simplified = expr.simplify(&["21122".to_string()]);
        assert_eq!(simplified, Some(Expr::Course("15122".to_string())));

        // Both completed
        let simplified = expr.simplify(&["15122".to_string(), "21122".to_string()]);
        assert_eq!(simplified, None);
    }

    #[test]
    fn test_simplify_or() {
        // Test OR expression
        let expr = Expr::Or(
            Box::new(Expr::Course("15122".to_string())),
            Box::new(Expr::Course("21122".to_string())),
        );

        // Nothing completed
        let simplified = expr.simplify(&[]);
        assert_eq!(simplified, Some(expr.clone()));

        // First course completed
        let simplified = expr.simplify(&["15122".to_string()]);
        assert_eq!(simplified, None);

        // Second course completed
        let simplified = expr.simplify(&["21122".to_string()]);
        assert_eq!(simplified, None);
    }

    #[test]
    fn test_simplify_complex() {
        // Test more complex expression
        let expr = Expr::Or(
            Box::new(Expr::And(
                Box::new(Expr::Course("15122".to_string())),
                Box::new(Expr::Course("21122".to_string())),
            )),
            Box::new(Expr::Course("15213".to_string())),
        );

        // One of the AND conditions met
        let simplified = expr.simplify(&["15122".to_string()]);
        assert_eq!(
            simplified,
            Some(Expr::Or(
                Box::new(Expr::Course("21122".to_string())),
                Box::new(Expr::Course("15213".to_string()))
            ))
        );

        // Both AND conditions met
        let simplified = expr.simplify(&["15122".to_string(), "21122".to_string()]);
        assert_eq!(simplified, None);

        // OR condition met
        let simplified = expr.simplify(&["15213".to_string()]);
        assert_eq!(simplified, None);
    }
}
