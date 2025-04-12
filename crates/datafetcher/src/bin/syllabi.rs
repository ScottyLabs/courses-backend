use datafetcher::util::{
    create_csv_writer, execute_hurl, get_captures, insert_variable, zip_captures,
};
use dotenv_codegen::dotenv;
use hurl::runner::VariableSet;
use models::syllabus_data::{Department, Season, Year};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde::Serialize;

/// Canvas API access token retrieved from environment variables
const CANVAS_ACCESS_TOKEN: &str = dotenv!("CANVAS_ACCESS_TOKEN");
/// Output file name
const OUTPUT_FILE: &str = "syllabi.csv";
/// CSV headers
const CSV_HEADERS: [&str; 5] = ["season", "year", "number", "section", "url"];

/// Struct representing a syllabus file with a URL
#[derive(Debug, Clone, Serialize)]
struct FileWithUrl {
    season: Season,
    year: Year,
    number: String,
    section: String,
    url: String,
}

/// Parses a title string to extract a course number and section.
///
/// This function handles various title formats, including:
/// - "S18 12345" pattern (returns "12345" as course number)
/// - Course number/section split by hyphens (e.g., "15122-A" returns "15122" and "A")
/// - Multi-hyphen titles (e.g. "15-122-A" returns "15122" and "A")
/// - Titles with additional information after the section (e.g., "15122-A: Syllabus")
/// - Titles that are actually a filename (e.g., "15122-A.pdf")
///
/// # Arguments
/// * `title` - The raw title string to parse
///
/// # Returns
/// A tuple of (course_number, section)
fn parse_title(title: &str) -> (String, String) {
    // Handle "S18 XXXXX" pattern
    if let Some(idx) = title.find("S18 ") {
        let after = &title[idx + 4..];
        if after.len() >= 5 && after[..5].chars().all(|c| c.is_ascii_digit()) {
            return (after[..5].to_string(), String::new());
        }
    }

    // Remove everything before the first digit and after the relevant information
    let title = match title.find(|c: char| c.is_ascii_digit()) {
        Some(idx) => &title[idx..],
        None => return ("unknown".to_string(), "".to_string()),
    };
    let title = title.split([' ', '.', '_', ':']).next().unwrap_or(title);

    // Split by "-" and analyze the parts
    match title.split('-').collect::<Vec<_>>().as_slice() {
        [] => ("unknown".into(), String::new()),
        [course_num] => (course_num.to_string(), String::new()),
        [course_num, section] if course_num.len() == 5 => {
            (course_num.to_string(), section.to_string())
        }
        [first, second] => (format!("{}{}", first, second), String::new()),
        [course_num, label, _]
            if matches!(label.to_lowercase().as_str(), "objectives" | "syllabus") =>
        {
            (course_num.to_string(), String::new())
        }
        [first, second, section] => (format!("{}{}", first, second), section.to_string()),
        [first, second, rest @ ..] => (format!("{}{}", first, second), rest.join("-")),
    }
}

/// Processes a department to retrieve file URLs for a given season and year
///
/// Makes HTTP requests to retrieve all syllabus files for a specified department,
/// season, and year combination. Parses the response to extract the file URLs and titles.
///
/// # Arguments
/// * `department` - The department to process
/// * `season` - The season (Fall, Spring, etc.)
/// * `year` - The academic year
/// * `vars` - The base variable set with tokens
/// * `file_urls_script` - The Hurl script to run for retrieving file URLs
///
/// # Returns
/// `Some(Vec<FileWithUrl>)` if successful, `None` if failed
fn process_department(
    department: Department,
    season: Season,
    year: Year,
    vars: &VariableSet,
    file_urls_script: &str,
) -> Option<Vec<FileWithUrl>> {
    // Create a new variable set with department, season, and year variables
    let mut group_vars = vars.clone();
    insert_variable(&mut group_vars, "department", department.into());
    insert_variable(&mut group_vars, "season", season.as_str());
    insert_variable(&mut group_vars, "year", &year.to_string());

    // Run the Hurl script to get file URLs
    let result = match execute_hurl(file_urls_script, &group_vars) {
        Ok(result) if result.success => result,
        Ok(_) => return None,
        Err(e) => {
            eprintln!(
                "Failed to run file URLs script for department {}: {}",
                department, e
            );
            return None;
        }
    };

    // Extract file URLs and titles from captures
    let files = zip_captures(&result, "file_urls", "title", |(url, title)| {
        let (number, section) = parse_title(&title.to_string());
        // Filter out entries with unknown course numbers or empty URLs
        (number != "unknown" && url.to_string().trim().is_empty()).then_some(FileWithUrl {
            number,
            section,
            season,
            year,
            url: url.to_string(),
        })
    });

    if !files.is_empty() {
        println!("Finished: {}{}-{}", season.as_str(), year, department);
        Some(files)
    } else {
        None
    }
}

/// Constructs a final FileWithUrl with the actual download URL
///
/// Takes an initial FileWithUrl with a reference URL and runs a Hurl request
/// to get the final download URL.
///
/// # Arguments
/// * `file` - The initial FileWithUrl object
/// * `vars` - The base variable set with tokens
/// * `final_url_script` - The Hurl script to run
///
/// # Returns
/// `Some(FileWithUrl)` with the final URL if successful, `None` otherwise
fn get_final_file(
    file: FileWithUrl,
    vars: &VariableSet,
    final_url_script: &str,
) -> Option<FileWithUrl> {
    let mut file_vars = vars.clone();
    insert_variable(&mut file_vars, "file_url", &file.url);

    match execute_hurl(final_url_script, &file_vars) {
        Ok(result) if result.success => get_captures(&result).first().and_then(|final_url| {
            let url = final_url.value.to_string();
            (!url.trim().is_empty()).then_some(FileWithUrl { url, ..file })
        }),
        _ => None,
    }
}

/// Generates all combinations of Department, Season, and Year
///
/// # Returns
/// A vector of (Department, Season, Year) tuples
fn generate_combinations() -> Vec<(Department, Season, Year)> {
    let mut combinations = Vec::new();

    for department in Department::all() {
        for season in Season::all() {
            for year in Year::all() {
                combinations.push((department, season, year));
            }
        }
    }

    combinations
}

/// Orchestrates the scraping of syllabus files
fn main() {
    // Load Hurl scripts
    let file_urls_script = include_str!("../../scripts/file_urls.hurl");
    let final_url_script = include_str!("../../scripts/final_url.hurl");

    // Set up variables for API authentication
    let mut vars = VariableSet::new();
    insert_variable(&mut vars, "token", CANVAS_ACCESS_TOKEN);

    // Initialize CSV writer for output
    let mut csv_writer = create_csv_writer(OUTPUT_FILE, &CSV_HEADERS).unwrap();

    // Process all department/season/year combinations in parallel
    let records = generate_combinations()
        .into_par_iter()
        // Get initial file URLs for each department/season/year
        .filter_map(|(department, season, year)| {
            process_department(department, season, year, &vars, file_urls_script)
        })
        .flatten()
        // Get final download URLs for each file
        .filter_map(|file| get_final_file(file, &vars, final_url_script))
        // Format each file as a CSV record
        .map(|file| {
            vec![
                file.season.as_str().to_string(),
                file.year.to_string(),
                file.number,
                file.section,
                file.url,
            ]
        })
        .collect::<Vec<_>>();

    // Write all records to CSV
    for record in records {
        csv_writer
            .write_record(record)
            .expect("Failed to write CSV record");
    }
    csv_writer.flush().expect("Failed to flush CSV writer");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_title() {
        let title = "02701-A: CPCB Course - Current Topics in Computational Biology";
        assert_eq!(parse_title(title), ("02701".to_string(), "A".to_string()));

        let title = "14513-syllabus-f18.pdf";
        assert_eq!(parse_title(title), ("14513".to_string(), "".to_string()));

        let title = "14809- Introduction to Cyber Intelligence.pdf";
        assert_eq!(parse_title(title), ("14809".to_string(), "".to_string()));

        let title = "14815 Syllabus.docx";
        assert_eq!(parse_title(title), ("14815".to_string(), "".to_string()));

        let title = "49-747_InnovationMindsetinPractice_Ayoob_E_Bodily_B.docx";
        assert_eq!(parse_title(title), ("49747".to_string(), "".to_string()));

        let title = "CMUiii_MIIPS Online 49-600_Syllabus.pdf";
        assert_eq!(parse_title(title), ("49600".to_string(), "".to_string()));

        let title = "85314.docx";
        assert_eq!(parse_title(title), ("85314".to_string(), "".to_string()));

        let title = "98317-A: Student Taught Courses (StuCo): Hype for Types";
        assert_eq!(parse_title(title), ("98317".to_string(), "A".to_string()));
    }

    #[test]
    fn test_generate_combinations() {
        // Ensure the function generates all expected combinations
        let combinations = generate_combinations();

        // Check if we have the expected number of combinations
        let expected_count = Department::all().len() * Season::all().len() * Year::all().len();
        assert_eq!(combinations.len(), expected_count);

        // Verify some specific combinations are present
        assert!(combinations.contains(&(Department::CS, Season::Fall, Year(2020))));
        assert!(combinations.contains(&(Department::MSC, Season::Spring, Year(2019))));
    }
}
