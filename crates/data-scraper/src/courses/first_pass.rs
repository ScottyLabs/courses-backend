/// Discard the header rows in the SOC file before parsing
///
/// # Arguments
/// * `input` - A raw string slice containing the full input file contents
///
/// # Returns
/// An iterator over the cleaned, line-based content of the SOC input
fn preprocess_lines(input: &str) -> impl Iterator<Item = &str> {
    input.lines().skip(11).map(str::trim_end)
}

/// Determines whether a string is a valid course number.
///
/// # Arguments
/// * `s` - A string slice to check.
///
/// # Returns
/// `true` if the input is a 5-digit numeric string (e.g., `"15122"`), `false` otherwise.
fn is_course_number(s: &str) -> bool {
    s.len() == 5 && s.chars().all(|c| c.is_ascii_digit())
}

/// Determines whether a string looks like a valid section code.
///
/// # Arguments
/// * `s` - A string slice to check.
///
/// # Returns
/// `true` if the input starts with an uppercase ASCII letter, `false` otherwise.
fn is_section_code(s: &str) -> bool {
    let first = s.chars().next();
    first.map_or(false, |c| c.is_ascii_uppercase())
}

/// Determines which `Line` variant a single line fits into
///
/// # Arguments
/// * `line` - A line of text from the input file
///
/// # Returns
/// * A `Line` enum variant representing the type of line
fn parse_line(line: &str) -> Line {
    let trimmed = line.trim();

    if trimmed.is_empty() {
        return Line::Empty;
    }

    // Fields are split by tabs in the schedule of classes
    let fields: Vec<&str> = trimmed.split('\t').collect();
    let leading_tabs = line.chars().take_while(|&c| c == '\t').count();

    match fields.as_slice() {
        // Department line: Only a department name
        [only] if leading_tabs == 1 => Line::Department(only.to_string()),

        // CourseHeader: number + title only
        [number, title] if is_course_number(number) => Line::CourseHeader {
            number: number.to_string(),
            title: title.trim().to_string(),
        },

        // SecondaryCourseHeader: number + title + units
        [number, title, units, ..]
        if is_course_number(number) && Units::from_str(units).is_ok() =>
            {
                Line::SecondaryCourseHeader {
                    number: number.to_string(),
                    title: title.trim().to_string(),
                    units: units.to_string(),
                }
            }

        // PrimaryCourseComponent: starts with units
        [
        units,
        section,
        days,
        time_start,
        time_end,
        building_room,
        location,
        instructors,
        ..,
        ] if Units::from_str(units).is_ok() => Line::PrimaryCourseComponent {
            units: units.to_string(),
            section: section.to_string(),
            days: days.to_string(),
            time_start: time_start.to_string(),
            time_end: time_end.to_string(),
            building_room: building_room.to_string(),
            location: location.to_string(),
            instructors: instructors.to_string(),
        },

        // SecondaryCourseComponent: starts with section
        [
        section,
        days,
        time_start,
        time_end,
        building_room,
        location,
        instructors,
        ..,
        ] if is_section_code(section) => Line::SecondaryCourseComponent {
            section: section.to_string(),
            days: days.to_string(),
            time_start: time_start.to_string(),
            time_end: time_end.to_string(),
            building_room: building_room.to_string(),
            location: location.to_string(),
            instructors: instructors.to_string(),
        },

        // AdditionalMeeting: days + times + building + location
        [days, time_start, time_end, building_room, location, ..] => Line::AdditionalMeeting {
            days: days.to_string(),
            time_start: time_start.to_string(),
            time_end: time_end.to_string(),
            building_room: building_room.to_string(),
            location: location.to_string(),
        },

        // ComponentTitle: short string that doesn't match other formats
        [title] if leading_tabs == 2 => Line::ComponentTitle(title.to_string()),

        // Unknown: matches none of the above, log for diagnostics
        _ => {
            println!("Unknown line format: {}", line);
            Line::Unknown(line.to_string())
        }
    }
}

/// Converts raw SOC input text into a flat list of structured `Line` variants.
///
/// This function skips the initial header rows using [`preprocess_lines`] and then parses
/// each subsequent line using [`parse_line`] to classify it into one of the variants of the
/// [`Line`] enum (e.g., `CourseHeader`, `PrimaryCourseComponent`, etc.).
///
/// # Arguments
/// * `input` - A raw string slice containing the full SOC file contents.
///
/// # Returns
/// A `Vec<Line>` representing all non-header lines, each classified into its appropriate variant.
pub fn first_pass(input: &str) -> Vec<Line> {
    preprocess_lines(input).map(parse_line).collect()
}