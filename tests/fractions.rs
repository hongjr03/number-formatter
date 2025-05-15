use number_format::types::LocaleSettings;
use number_format::{format_number, parse_number_format};

fn fmt(value: f64, pattern: &str) -> String {
    let fmt = parse_number_format(pattern).unwrap_or_else(|e| {
        panic!("Failed to parse pattern '{}': {}", pattern, e);
    });
    format_number(value, &fmt, &LocaleSettings::default())
}

#[test]
fn test_simple_fractions() {
    assert_eq!(fmt(5.25, "# ?/?"), "5 1/4");
    assert_eq!(fmt(5.75, "# ?/?"), "5 3/4");
    assert_eq!(fmt(5.5, "# ?/?"), "5 1/2");
    assert_eq!(fmt(0.5, "?/?"), "1/2");
    assert_eq!(fmt(0.5, "0 ?/?"), "0 1/2");
    assert_eq!(fmt(0.0, "?/?"), "0/1");
    assert_eq!(fmt(1.0 / 3.0, "# ?/?"), " 1/3");
    assert_eq!(fmt(2.0 / 3.0, "# ?/?"), " 2/3");
    assert_eq!(fmt(1.0, "# ?/?"), "1   ");
}

#[test]
fn test_fraction_placeholders_and_alignment() {
    assert_eq!(fmt(5.0625, "# ??/??"), "5  1/16");
    assert_eq!(fmt(5.125, "# ??/??"), "5  1/8 ");

    assert_eq!(fmt(0.0625, "??/??"), " 1/16");
    assert_eq!(fmt(0.125, "??/??"), " 1/8 ");

    assert_eq!(fmt(5.3, "# ?/??"), "5 3/10");
    assert_eq!(fmt(0.3, "?/??"), "3/10");
    assert_eq!(fmt(0.3, "??/??"), " 3/10");
}

#[test]
fn test_whole_numbers_with_fraction_format() {
    assert_eq!(fmt(5.0, "# ?/?"), "5   ");
    assert_eq!(fmt(5.0, "0 ?/?"), "5   ");
    assert_eq!(fmt(123.0, "# ?/?"), "123   ");
}

#[test]
fn test_mixed_numbers_various_denominators() {
    assert_eq!(fmt(2.5, "# ?/?"), "2 1/2");
    assert_eq!(fmt(2.125, "# ?/???"), "2 1/8  ");
    assert_eq!(fmt(2.0625, "# ??/???"), "2  1/16 ");
}

#[test]
fn test_zero_value_formatting_exact() {
    assert_eq!(fmt(0.0, "?/?"), "0/1");
    assert_eq!(fmt(0.0, "??/??"), " 0/1 ");
    assert_eq!(fmt(0.0, "???/???"), "  0/1  ");
    assert_eq!(fmt(0.0, "# ?/?"), "0   ");
    assert_eq!(fmt(0.0, "0 ?/?"), "0   ");
}
