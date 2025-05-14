use number_format::types::{FormatToken, LocaleSettings};
use number_format::{format_number, parse_number_format};

#[test]
fn test_basic_format() {
    let format = parse_number_format("0.00").unwrap();
    let locale = LocaleSettings::default();
    assert_eq!(format_number(123.456, &format, &locale), "123.46");
    assert_eq!(format_number(0.789, &format, &locale), "0.79");
}

#[test]
fn test_negative_format() {
    let format = parse_number_format("0.00;-0.00").unwrap();
    let locale = LocaleSettings::default();
    assert_eq!(format_number(123.456, &format, &locale), "123.46");
    assert_eq!(format_number(-123.456, &format, &locale), "-123.46");
}

#[test]
fn test_digit_placeholders() {
    let format = parse_number_format("#0.0#").unwrap();
    let locale = LocaleSettings::default();
    assert_eq!(format_number(123.456, &format, &locale), "123.46");
    assert_eq!(format_number(123.4, &format, &locale), "123.4");
    assert_eq!(format_number(0.456, &format, &locale), "0.46");
}

#[test]
fn test_percent_format() {
    let format = parse_number_format("0%").unwrap();
    let locale = LocaleSettings::default();
    assert_eq!(format_number(0.12, &format, &locale), "12%");
}

#[test]
fn test_large_integers() {
    let format = parse_number_format("0.00").unwrap();
    let locale = LocaleSettings::default();
    assert_eq!(format_number(1234567.89, &format, &locale), "1234567.89");

    let format2 = parse_number_format("0").unwrap();
    assert_eq!(format_number(12345.0, &format2, &locale), "12345");
}

#[test]
fn test_rounding() {
    let format = parse_number_format("0.0").unwrap();
    let locale = LocaleSettings::default();
    assert_eq!(format_number(0.04, &format, &locale), "0.0");
    assert_eq!(format_number(0.05, &format, &locale), "0.1");
    assert_eq!(format_number(0.95, &format, &locale), "1.0");

    let format2 = parse_number_format("0.00").unwrap();
    assert_eq!(format_number(0.994, &format2, &locale), "0.99");
    assert_eq!(format_number(0.995, &format2, &locale), "1.00");
}

#[test]
fn test_zero_format() {
    let format = parse_number_format("0.00;-0.00;\"零\"").unwrap();
    let locale = LocaleSettings::default();
    assert_eq!(format_number(0.0, &format, &locale), "零");
}

#[test]
fn test_conditional_formats() {
    let format = parse_number_format("[>100]\"大数字\"").unwrap();
    let locale = LocaleSettings::default();
    assert_eq!(format_number(150.0, &format, &locale), "大数字");

    let format2 = parse_number_format("[<=100]\"小数字\"").unwrap();
    assert_eq!(format_number(50.0, &format2, &locale), "小数字");
}

#[test]
fn test_thousands_separator_default_locale() {
    let format = parse_number_format("#,##0.00").unwrap();
    let locale = LocaleSettings::default();
    assert_eq!(format_number(1234567.89, &format, &locale), "1,234,567.89");
    assert_eq!(format_number(1234.56, &format, &locale), "1,234.56");
    assert_eq!(format_number(123.45, &format, &locale), "123.45");
    assert_eq!(format_number(0.12, &format, &locale), "0.12");
    assert_eq!(format_number(-12345.67, &format, &locale), "-12,345.67");
}

#[test]
fn test_thousands_separator_localized() {
    let format = parse_number_format("#,##0.00").unwrap();
    let custom_locale = LocaleSettings {
        decimal_point: ',',
        thousands_separator: '.',
    };
    assert_eq!(
        format_number(1234567.89, &format, &custom_locale),
        "1.234.567,89"
    );
    assert_eq!(
        format_number(-12345.67, &format, &custom_locale),
        "-12.345,67"
    );

    let underscore_locale = LocaleSettings {
        decimal_point: '.',
        thousands_separator: '_',
    };
    assert_eq!(
        format_number(1234567.89, &format, &underscore_locale),
        "1_234_567.89"
    );
}

#[test]
fn test_no_thousands_separator_if_format_lacks_comma() {
    let format = parse_number_format("0.00").unwrap();
    let locale_with_separator = LocaleSettings {
        decimal_point: '.',
        thousands_separator: '_',
    };
    assert_eq!(
        format_number(12345.67, &format, &locale_with_separator),
        "12345.67"
    );

    let format_hash = parse_number_format("###0.00").unwrap();
    assert_eq!(
        format_number(12345.67, &format_hash, &locale_with_separator),
        "12345.67"
    );
}

#[test]
fn test_different_digit_placeholders() {
    let format1 = parse_number_format("###0.0#").unwrap();
    let format2 = parse_number_format("0000.0#").unwrap();
    let format3 = parse_number_format("???0.0#").unwrap();
    let locale = LocaleSettings::default();

    assert_eq!(format_number(123.45, &format1, &locale), "123.45");
    assert_eq!(format_number(123.45, &format2, &locale), "0123.45");
    assert_eq!(format_number(123.45, &format3, &locale), " 123.45");
}

#[test]
fn test_parentheses_for_negative() {
    let format = parse_number_format("0.00;(0.00)").unwrap();
    let locale = LocaleSettings::default();
    assert_eq!(format_number(123.45, &format, &locale), "123.45");
    assert_eq!(format_number(-123.45, &format, &locale), "(123.45)");
}

#[test]
fn test_text_and_numbers() {
    let format = parse_number_format("\"价格：\"0.00\" 元\"").unwrap();
    let locale = LocaleSettings::default();
    assert_eq!(format_number(123.45, &format, &locale), "价格：123.45 元");
}

#[test]
fn test_exponential_format() {
    let format = parse_number_format("0.00E+00").unwrap();
    let locale = LocaleSettings::default();
    assert_eq!(format_number(12345.67, &format, &locale), "1.23E+04");
    assert_eq!(format_number(0.00012345, &format, &locale), "1.23E-04");
}

#[test]
fn test_edge_cases() {
    let format = parse_number_format("0.00").unwrap();
    let locale = LocaleSettings::default();
    assert_eq!(
        format_number(1.0e18, &format, &locale),
        "1000000000000000000.00"
    );
    assert_eq!(format_number(1.0e-11, &format, &locale), "0.00");
    assert_eq!(format_number(-0.0, &format, &locale), "0.00");
}

#[test]
fn test_localized_decimal_point() {
    let format = parse_number_format("0.00").unwrap();
    let locale_de = LocaleSettings {
        decimal_point: ',',
        thousands_separator: '.',
    };
    assert_eq!(format_number(123.45, &format, &locale_de), "123,45");
    assert_eq!(
        format_number(0.995, &parse_number_format("0.00").unwrap(), &locale_de),
        "1,00"
    );
}

#[test]
fn test_scaling_commas() {
    let default_locale = LocaleSettings::default();

    // Simple scaling
    let format_simple = parse_number_format("0,").unwrap();
    assert_eq!(format_simple.positive_section.num_scaling_commas, 1);
    assert_eq!(
        format_simple.positive_section.tokens,
        vec![FormatToken::DigitOrZero]
    );
    assert_eq!(
        format_number(12345.0, &format_simple, &default_locale),
        "12"
    );
    assert_eq!(
        format_number(12789.0, &format_simple, &default_locale),
        "13"
    );

    // Multiple scaling commas
    let format_double = parse_number_format("0,,").unwrap();
    assert_eq!(format_double.positive_section.num_scaling_commas, 2);
    assert_eq!(
        format_double.positive_section.tokens,
        vec![FormatToken::DigitOrZero]
    );
    assert_eq!(
        format_number(12345678.0, &format_double, &default_locale),
        "12"
    );
    assert_eq!(
        format_number(12789123.0, &format_double, &default_locale),
        "13"
    );

    // Scaling with decimals in format
    let format_decimal_scale = parse_number_format("0.0,").unwrap();
    assert_eq!(format_decimal_scale.positive_section.num_scaling_commas, 1);
    assert_eq!(
        format_decimal_scale.positive_section.tokens,
        vec![
            FormatToken::DigitOrZero,
            FormatToken::DecimalPoint,
            FormatToken::DigitOrZero
        ]
    );
    assert_eq!(
        format_number(12345.6, &format_decimal_scale, &default_locale),
        "12.3"
    );
    assert_eq!(
        format_number(123.4, &format_decimal_scale, &default_locale),
        "0.1"
    );

    // Scaling with text
    let format_clear_scale_text = parse_number_format("0,,\"M\"").unwrap();
    assert_eq!(
        format_clear_scale_text.positive_section.num_scaling_commas,
        2
    );
    assert_eq!(
        format_clear_scale_text.positive_section.tokens,
        vec![
            FormatToken::DigitOrZero,
            FormatToken::QuotedText("M".to_string()),
        ]
    );
    assert_eq!(
        format_number(12345678.0, &format_clear_scale_text, &default_locale),
        "12M"
    );

    // Scaling for zero value
    assert_eq!(format_number(0.0, &format_simple, &default_locale), "0");

    // Scaling for negative value
    let format_neg_simple = parse_number_format("0,;-0,").unwrap();
    assert_eq!(
        format_number(-12345.0, &format_neg_simple, &default_locale),
        "-12"
    );
    assert_eq!(
        format_number(-12789.0, &format_neg_simple, &default_locale),
        "-13"
    );

    // Scaling combined with thousands separator
    let format_combo = parse_number_format("#,##0.0, \"K\"").unwrap();
    assert_eq!(format_combo.positive_section.num_scaling_commas, 1);
    assert_eq!(
        format_combo.positive_section.tokens,
        vec![
            FormatToken::DigitIfNeeded,
            FormatToken::ThousandsSeparator,
            FormatToken::DigitIfNeeded,
            FormatToken::DigitIfNeeded,
            FormatToken::DigitOrZero,
            FormatToken::DecimalPoint,
            FormatToken::DigitOrZero,
            FormatToken::LiteralChar(' '),
            FormatToken::QuotedText("K".to_string()),
        ]
    );
    assert_eq!(
        format_number(1234567.89, &format_combo, &default_locale),
        "1,234.6 K"
    );
    assert_eq!(
        format_number(567.0, &format_combo, &default_locale),
        "0.6 K"
    );

    // Format with only scaling commas (no other numeric placeholders)
    let format_only_commas = parse_number_format(",,").unwrap();
    assert_eq!(
        format_only_commas.positive_section.num_scaling_commas, 2,
        "Test: ,, format - num_scaling_commas"
    );
    assert!(
        format_only_commas.positive_section.tokens.is_empty(),
        "Test: ,, format - tokens empty"
    );
    // assert_eq!(
    //     format_number(12345678.0, &format_only_commas, &default_locale),
    //     ""
    // );

    // Test a case where comma is clearly part of text and not scaling
    let format_text_comma = parse_number_format("\"Total: ,\"0").unwrap();
    assert_eq!(format_text_comma.positive_section.num_scaling_commas, 0);
    assert_eq!(
        format_text_comma.positive_section.tokens,
        vec![
            FormatToken::QuotedText("Total: ,".to_string()),
            FormatToken::DigitOrZero
        ]
    );
    assert_eq!(
        format_number(123.0, &format_text_comma, &default_locale),
        "Total: ,123"
    );

    // Test literal comma, then number, then scaling comma.
    let format_literal_comma_then_scale = parse_number_format("\",\"0.0,").unwrap();
    assert_eq!(
        format_literal_comma_then_scale
            .positive_section
            .num_scaling_commas,
        1
    );
    assert_eq!(
        format_literal_comma_then_scale.positive_section.tokens,
        vec![
            FormatToken::QuotedText(",".to_string()),
            FormatToken::DigitOrZero,
            FormatToken::DecimalPoint,
            FormatToken::DigitOrZero,
        ]
    );
    assert_eq!(
        format_number(4567.0, &format_literal_comma_then_scale, &default_locale),
        ",4.6"
    );
}
