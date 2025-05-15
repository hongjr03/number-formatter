#![allow(unused)]

use number_format::types::LocaleSettings;
use number_format::{format_number, parse_number_format};
use serde::Deserialize;
use std::fs;
use std::io::{self, Write};

#[derive(Debug, Deserialize)]
struct TestCase {
    value: serde_json::Value,
    format: String,
    expected: String,
    name: Option<String>,
    comment: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TestCases {
    cases: Vec<TestCase>,
}

#[test]
#[cfg(feature = "run_excelize_tests")]
fn run_all_excelize_toml_cases() {
    let toml_content = fs::read_to_string("tests/excelize-numfmt-test.toml")
        .expect("Failed to read tests/excelize-numfmt-test.toml");
    let test_data: TestCases = toml::from_str(&toml_content)
        .expect("Failed to parse TOML from tests/excelize-numfmt-test.toml");

    let mut passed_count = 0;
    let mut failed_count = 0;
    let total_count = test_data.cases.len();

    // Assuming default locale settings are sufficient or can be configured as needed.
    // The summary mentioned `LocaleSettings::default_for_currency_symbol("¤")` was used before.
    // Let's use a default or a specific one for consistency if needed.
    // For now, if `format_number` takes Option<&LocaleSettings>, None might be okay for some tests,
    // but currency tests might need specific locale.
    // The function signature from summary: format_number(value: f64, format_str: &str, text_value: Option<&str>, locale: Option<&LocaleSettings>) -> String
    // Let's stick to None for locale if not specified or make it configurable if tests require it.
    // For the `¤` issue, a specific locale was key. Let's use a default that allows `¤` to work if possible.
    let locale_settings_default = LocaleSettings::default();
    for case in test_data.cases.iter() {
        let num_value = match &case.value {
            serde_json::Value::Number(n) => n.as_f64().unwrap_or(f64::NAN),
            serde_json::Value::String(s) if s.to_lowercase() == "nan" => f64::NAN,
            serde_json::Value::String(s)
                if s.to_lowercase() == "inf" || s.to_lowercase() == "+inf" =>
            {
                f64::INFINITY
            }
            serde_json::Value::String(s) if s.to_lowercase() == "-inf" => f64::NEG_INFINITY,
            // If it's a string that can be parsed as a number, attempt that.
            serde_json::Value::String(s) => s.parse::<f64>().unwrap_or(f64::NAN), // Or handle text value differently
            _ => f64::NAN, // Default for other types or if strict numeric context
        };

        let format = parse_number_format(&case.format)
            .map_err(|e| {
                eprintln!("Error parsing format: {}", e);
                "N/A"
            });

        // if format cannot be unwrapped, skip the test and increment the failed count
        if !format.is_ok() {
            failed_count += 1;
            continue;
        }

        let actual_result = format_number(
            num_value,
            &format.unwrap(),
            // Pass text_value, which could be the original string if value was a string.
            &locale_settings_default,
        );

        if actual_result == case.expected {
            passed_count += 1;
        } else {
            failed_count += 1;
            eprintln!(
                "Test FAILED: Name: '{}', Comment: '{}'\n  Format: '{}'\n  Value: {:?}\n  Expected: '{}'\n  Actual:   '{}'",
                case.name.as_deref().unwrap_or("N/A"),
                case.comment.as_deref().unwrap_or("N/A"),
                case.format,
                case.value,
                case.expected,
                actual_result
            );
        }
    }

    // Ensure this output is exactly what the CI script expects to parse.
    println!(
        "PASSED:{}\nFAILED:{}\nTOTAL:{}",
        passed_count, failed_count, total_count
    );

    // Flush stdout to ensure output is written before panic or exit.
    io::stdout().flush().unwrap();

    if failed_count > 0 {
        panic!(
            "{} of {} tests failed from excelize-numfmt-test.toml",
            failed_count, total_count
        );
    }
}
