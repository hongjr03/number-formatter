#[cfg(test)]
mod tests {
    use number_format::types::LocaleSettings;
    use number_format::{format_number, parse_number_format};
    use serde::Deserialize;
    use std::fs;
    use std::path::{Path, PathBuf};

    #[derive(Debug, Deserialize)]
    struct TestCase {
        #[allow(dead_code)]
        value: f64,
        #[allow(dead_code)]
        format: String,
        #[allow(dead_code)]
        expected: String,
    }

    #[derive(Debug, Deserialize)]
    struct TestCases {
        #[allow(dead_code)]
        cases: Vec<TestCase>,
    }

    #[allow(dead_code)]
    fn run_test_case(case: &TestCase, locale_settings: &LocaleSettings) -> Result<(), String> {
        let format = parse_number_format(&case.format)
            .map_err(|e| format!("Format parse error: {:?}", e))?;

        let result = format_number(case.value, &format, locale_settings);

        if result != case.expected {
            return Err(format!(
                "\n✗ Mismatch for value: {}\nFormat:     \"{}\"\nExpected:   \"{}\"\nActual:     \"{}\"",
                case.value, case.format, case.expected, result
            ));
        }

        Ok(())
    }

    #[allow(dead_code)]
    fn main() {
        let toml_path: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("excelize-numfmt-test.toml");

        let toml_content = fs::read_to_string(&toml_path)
            .unwrap_or_else(|e| panic!("Failed to read TOML file {}: {}", toml_path.display(), e));

        let test_suite: TestCases = toml::from_str(&toml_content)
            .unwrap_or_else(|e| panic!("Failed to parse TOML file {}: {}", toml_path.display(), e));

        let default_locale = LocaleSettings::default();
        let mut passed = 0;
        let mut failed = 0;

        for (i, case) in test_suite.cases.iter().enumerate() {
            match run_test_case(case, &default_locale) {
                Ok(_) => passed += 1,
                Err(msg) => {
                    failed += 1;
                    // Print immediately for CI logs if --nocapture is used
                    eprintln!("\n[Case {}] {}", i + 1, msg);
                }
            }
        }

        // Final summary output
        println!(
            "{{ \"passed\": {}, \"failed\": {}, \"total\": {} }}", // JSON output for easier parsing
            passed,
            failed,
            passed + failed
        );

        if failed > 0 {
            // Exit with a non-zero status code if any tests failed
            std::process::exit(1);
        }
    }
}
