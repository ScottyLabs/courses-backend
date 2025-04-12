use data_scraper::courses::first_pass::first_pass;
use data_scraper::courses::second_pass::second_pass;
use futures::future::join_all;
use models::syllabus_data::{Season, Year};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use reqwest::Client;

/// Retrieves the year from the course data text
///
/// # Arguments
/// * `text` - A string slice containing the course data
///
/// # Returns
/// * An `Option<Year>` which is the year extracted from the text
fn extract_year(text: &str) -> Option<Year> {
    let line = text.lines().nth(3)?;
    let year_str = line.trim().split_whitespace().last()?;

    year_str.parse::<u16>().ok().map(Year)
}

/// Orchestrates the scraping of course details
#[tokio::main]
async fn main() {
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
        .collect::<Vec<_>>();

    let output = format!("{:#?}", results);
    std::fs::write("output.txt", output).expect("Unable to write file");
}
