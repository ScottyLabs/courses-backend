use datafetcher::{
    courses::{first_pass::first_pass, second_pass::second_pass},
    util::{DEFAULT_OUTPUT_DIR, ensure_dir},
};
use futures::future::join_all;
use models::syllabus_data::{Season, Year};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use reqwest::Client;
use std::{fs::File, io::Write, path::Path};

/// Output file name
const OUTPUT_FILE: &str = "courses.txt";

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

    // Download all in parallel
    let downloaded: Vec<(Season, String)> = join_all(futures).await;

    // Do parsing in parallel
    let results = downloaded
        .into_par_iter()
        .map(|(season, text)| {
            let year = extract_year(&text)
                .unwrap_or_else(|| panic!("Failed to extract year for {:?}", season));
            let lines = first_pass(&text);

            second_pass(lines, season, year)
        })
        .flatten()
        .collect::<Vec<_>>();

    let output = format!("{:#?}", results);
    Write::write_all(&mut file, output.as_bytes()).expect("Failed to write to file");
}
