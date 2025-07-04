use datafetcher::{
    courses::{first_pass::first_pass, second_pass::second_pass},
    util::{
        DEFAULT_OUTPUT_DIR, ensure_dir, execute_hurl, get_capture_value, get_captures,
        get_optional_string_value, get_parsed_struct_value, insert_variable, parse_from_raw_html,
    },
};
use futures::future::join_all;
use hurl::runner::VariableSet;
use models::{
    course_data::{CourseEntry, CourseMetadata, CourseObject},
    reservation::{Reservation, Restriction},
    syllabus_data::{Season, Year},
};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use reqwest::Client;
use scraper::{Html, Selector};
use std::{collections::HashMap, fs::File, io::Write, path::Path, str::FromStr};

/// Output file name
const OUTPUT_FILE: &str = "courses.txt";

/// Hurl script for retrieving course details
const COURSE_DETAILS_SCRIPT: &str = include_str!("../../scripts/course_details.hurl");

/// Retrieves the year from the course data text
///
/// # Arguments
/// * `text` - A string slice containing the course data
///
/// # Returns
/// * An `Option<Year>` which is the year extracted from the text
fn extract_year(text: &str) -> Option<Year> {
    let line = text.lines().nth(3)?;
    let year_str = line.split_whitespace().last()?;

    year_str.parse::<u16>().ok().map(Year)
}

/// Parses the related URLs from the full HTML document
///
/// # Arguments
/// * `document` - The [`Html`] document to parse
///
/// # Returns
/// A vector of related URLs as strings
fn parse_related_urls(document: &Html) -> Vec<String> {
    let selector = Selector::parse("#course-detail-related-urls a").unwrap();

    document
        .select(&selector)
        .filter_map(|link| link.value().attr("href"))
        .map(|href| href.to_owned())
        .collect()
}

/// Parses the reservations from the full HTML document
///
/// # Arguments
/// * `document` - The [`Html`] document to parse
///
/// # Returns
/// A vector of [`Reservation`] objects, each containing a section and its associated [`Restriction`]s
fn parse_reservations(document: &Html) -> Vec<Reservation> {
    let mut reservations_map: HashMap<String, Vec<Restriction>> = HashMap::new();

    let table_selector = Selector::parse("table").unwrap();
    let header_selector = Selector::parse("th").unwrap();

    let row_selector = Selector::parse("tr").unwrap();
    let cell_selector = Selector::parse("td").unwrap();

    // Find the table with the "Section" and "Restriction" headers
    for table in document.select(&table_selector) {
        let headers: Vec<String> = table
            .select(&header_selector)
            .map(|th| th.text().collect::<String>().trim().to_lowercase())
            .collect();

        if headers.contains(&"section".to_string()) && headers.contains(&"restriction".to_string())
        {
            // This is the reservations table, get the data rows
            for row in table.select(&row_selector) {
                let cells: Vec<_> = row.select(&cell_selector).collect();

                if cells.len() >= 2 {
                    let section = cells[0].text().collect::<String>().trim().to_owned();
                    let restriction_text = cells[1].text().collect::<String>().trim().to_owned();

                    if !section.is_empty() && !restriction_text.is_empty() {
                        let restriction =
                            Restriction::from_str(&restriction_text).unwrap_or_else(|e| {
                                eprintln!("Failed to parse restriction '{restriction_text}': {e}");
                                Restriction {
                                    student_type: None,
                                    restriction_type: None,
                                }
                            });

                        reservations_map
                            .entry(section)
                            .or_default()
                            .push(restriction);
                    }
                }
            }

            // This is the right table, stop looking
            break;
        }
    }

    reservations_map
        .into_iter()
        .map(|(section, restrictions)| Reservation {
            section,
            restrictions,
        })
        .collect()
}

/// Processes a course entry to get additional metadata for that course.
///
/// Makes an HTTP request to the courseDetails endpoint using the course number, season,
/// and year. Parses the response HTML with `xpath` to extract additional data.
///
/// # Arguments
/// * `course` - The course to process
///
/// # Returns
/// The [`CourseObject`] containing the full course object, with the `metadata` field
/// set to `Some(CourseMetadata)` if successful and `None` otherwise.
fn process_course_details(course: CourseEntry) -> CourseObject {
    // Create a new variable set with course, season, and year variables
    let mut vars = VariableSet::new();
    insert_variable(&mut vars, "course", &course.number.to_string());
    insert_variable(&mut vars, "season", course.season.as_str());
    insert_variable(&mut vars, "year", &course.year.to_string());

    // Run the Hurl script to get course details
    let result = match execute_hurl(COURSE_DETAILS_SCRIPT, &vars) {
        Ok(result) if result.success => result,
        Ok(_) => {
            return CourseObject {
                base_course: course,
                metadata: None,
            };
        }
        Err(e) => {
            eprintln!(
                "Failed to run course details script for course {} ({}{}): {e}",
                course.number.as_full_string(),
                course.season.as_str(),
                course.year
            );
            return CourseObject {
                base_course: course,
                metadata: None,
            };
        }
    };

    // Extract individual fields from result using get_capture_value
    let special_permission = get_capture_value(&result, "special_permission")
        .map(|v| matches!(v.to_string().trim().to_lowercase().as_str(), "yes"))
        .unwrap_or_default();

    // Create CourseMetadata with extracted information
    let captures = get_captures(&result);
    let raw_html = parse_from_raw_html(captures);

    let metadata = CourseMetadata {
        related_urls: parse_related_urls(&raw_html),
        special_permission,
        description: get_optional_string_value(&result, "description"),
        prerequisites: get_parsed_struct_value(&result, "prerequisites"),
        corequisites: get_parsed_struct_value(&result, "corequisites"),
        crosslisted: get_parsed_struct_value(&result, "crosslisted"),
        notes: get_optional_string_value(&result, "notes"),
        reservations: parse_reservations(&raw_html),
    };

    CourseObject {
        base_course: course,
        metadata: Some(metadata),
    }
}

/// Orchestrates the scraping of course details
#[tokio::main]
async fn main() {
    ensure_dir(DEFAULT_OUTPUT_DIR).unwrap();

    let path = Path::new(DEFAULT_OUTPUT_DIR).join(OUTPUT_FILE);
    let mut file = File::create(path).expect("Failed to create output file");

    let client = Client::new();

    // Build futures for downloading each season's data
    let futures = Season::all().into_iter().map(|season| {
        let client = client.clone();
        async move {
            let url = format!(
                "https://enr-apps.as.cmu.edu/assets/SOC/sched_layout_{}.dat",
                season.as_full_str()
            );

            let text = client
                .get(&url)
                .send()
                .await
                .expect("Request failed")
                .text()
                .await
                .expect("Failed to read body");

            (season, text)
        }
    });

    // Download all and parse in parallel
    let downloaded = join_all(futures).await;
    let results = downloaded
        .into_par_iter()
        .map(|(season, text)| {
            let year = extract_year(&text)
                .unwrap_or_else(|| panic!("Failed to extract year for {season:?}"));
            let lines = first_pass(&text);

            second_pass(lines, season, year)
        })
        .flatten()
        .into_par_iter()
        .map(process_course_details)
        .collect::<Vec<_>>();

    let output = format!("{results:#?}");
    Write::write_all(&mut file, output.as_bytes()).expect("Failed to write to file");
}
