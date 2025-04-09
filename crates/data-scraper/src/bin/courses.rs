use data_scraper::courses::first_pass::first_pass;
use data_scraper::courses::second_pass::second_pass;
use models::syllabus_data::{Season, Year};

/// Orchestrates the scraping of course details
fn main() {
    const SOC: &str = include_str!("../../data/input/soc.txt");
    let lines = first_pass(SOC);
    let result = second_pass(lines, Season::Fall, Year(2025));

    let output = format!("{:#?}", result);
    std::fs::write("output.txt", output).expect("Unable to write file");
}
