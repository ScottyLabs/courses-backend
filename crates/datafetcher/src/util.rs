use csv::Writer;
use hurl::{
    runner::{self, CaptureResult, HurlResult, RunnerOptionsBuilder, Value, VariableSet},
    util::logger::LoggerOptionsBuilder,
};
use std::{
    fs::{self, File},
    path::Path,
};

/// Output directory for data files
pub const DEFAULT_OUTPUT_DIR: &str = "./data/output";

/// Extracts all captures from a Hurl result
///
/// # Arguments
/// * `result` - The HurlResult containing captures
///
/// # Returns
/// A vector of references to CaptureResult
pub fn get_captures(result: &HurlResult) -> Vec<&CaptureResult> {
    result
        .entries
        .iter()
        .flat_map(|e| e.captures.iter())
        .collect()
}

/// Inserts a string variable into a VariableSet
///
/// # Arguments
/// * `vars` - The VariableSet to modify
/// * `key` - The name of the variable
/// * `value` - The string value to insert
///
/// # Panics
/// If the variable insertion fails
pub fn insert_variable(vars: &mut VariableSet, key: &str, value: &str) {
    vars.insert(key.to_string(), Value::String(value.to_string()))
        .unwrap_or_else(|_| panic!("Failed to insert variable: {key}"));
}

/// Executes a Hurl script with variables
///
/// # Arguments
/// * `script` - The Hurl script content
/// * `vars` - The variables to use
///
/// # Returns
/// Result containing HurlResult or error message
pub fn execute_hurl(script: &str, vars: &VariableSet) -> Result<HurlResult, String> {
    let runner_opts = RunnerOptionsBuilder::new().build();
    let logger_opts = LoggerOptionsBuilder::new().verbosity(None).build();

    runner::run(script, None, &runner_opts, vars, &logger_opts)
        .map_err(|e| format!("Hurl execution failed: {e}"))
}

/// Ensures a directory exists, creating it if necessary
///
/// # Arguments
/// * `dir_path` - Path to the directory
///
/// # Returns
/// Result indicating success or detailed error
pub fn ensure_dir(dir_path: &str) -> Result<(), String> {
    let path = Path::new(dir_path);
    if !path.exists() {
        fs::create_dir_all(path)
            .map_err(|e| format!("Failed to create directory '{dir_path}': {e}"))?;
    }

    Ok(())
}

/// Creates a CSV writer for the specified file
///
/// # Arguments
/// * `output_dir` - Directory to create the file in
/// * `filename` - Name of the CSV file
/// * `headers` - Column headers for the CSV
///
/// # Returns
/// Result containing the CSV writer or error message
pub fn create_csv_writer(filename: &str, headers: &[&str]) -> Result<Writer<File>, String> {
    ensure_dir(DEFAULT_OUTPUT_DIR)?;

    let path = Path::new(DEFAULT_OUTPUT_DIR).join(filename);
    let file =
        File::create(path).map_err(|e| format!("Failed to create CSV file '{filename}': {e}"))?;

    let mut writer = Writer::from_writer(file);
    writer
        .write_record(headers)
        .map_err(|e| format!("Failed to write CSV headers: {e}"))?;

    Ok(writer)
}

/// Safely gets a capture value from a Hurl result
///
/// # Arguments
/// * `result` - The HurlResult to extract from
/// * `capture_name` - Name of the capture to find
///
/// # Returns
/// Some(value) if found, None otherwise
pub fn get_capture_value<'a>(result: &'a HurlResult, capture_name: &'a str) -> Option<&'a Value> {
    get_captures(result)
        .iter()
        .find(|capture| capture.name == capture_name)
        .map(|capture| &capture.value)
}

/// Zips two capture lists together
///
/// # Arguments
/// * `result` - The HurlResult containing captures
/// * `list1_name` - Name of the first list capture
/// * `list2_name` - Name of the second list capture
///
/// # Returns
/// Vector of paired values if both lists exist and have the same length, empty vector otherwise
pub fn zip_captures<F, T>(
    result: &HurlResult,
    list1_name: &str,
    list2_name: &str,
    transform: F,
) -> Vec<T>
where
    F: Fn((&Value, &Value)) -> Option<T>,
{
    if let Some(Value::List(list1)) = get_capture_value(result, list1_name)
        && let Some(Value::List(list2)) = get_capture_value(result, list2_name)
        && list1.len() == list2.len()
    {
        return list1
            .iter()
            .zip(list2.iter())
            .filter_map(transform)
            .collect();
    }

    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_variable() {
        let mut vars = VariableSet::new();

        // Test successful insertion
        insert_variable(&mut vars, "test", "value");
        assert!(matches!(
            vars.get("test").map(|v| v.value()),
            Some(Value::String(_))
        ));
    }

    #[test]
    fn test_ensure_dir() {
        let test_dir = format!("{}/test_dir", std::env::temp_dir().to_string_lossy());
        let result = ensure_dir(&test_dir);
        assert!(result.is_ok());

        // Clean up
        let _ = fs::remove_dir(&test_dir);
    }
}
