use crate::parser::*;
use crate::types::*;

#[test]
fn test_simple_positive() {
    let result = parse_number_format("0.00").unwrap();
    assert_eq!(
        result.positive_section.tokens,
        vec![
            FormatToken::DigitOrZero,
            FormatToken::DecimalPoint,
            FormatToken::DigitOrZero,
            FormatToken::DigitOrZero,
        ]
    );
    assert!(result.negative_section.is_none());
}

#[test]
fn test_all_sections() {
    let result = parse_number_format("#,##0.00;[Red]-#,##0.00;0.00;\"Text: \"@").unwrap();
    assert_eq!(
        result.positive_section.tokens,
        vec![
            FormatToken::DigitIfNeeded,
            FormatToken::ThousandsSeparator,
            FormatToken::DigitIfNeeded,
            FormatToken::DigitIfNeeded,
            FormatToken::DigitOrZero,
            FormatToken::DecimalPoint,
            FormatToken::DigitOrZero,
            FormatToken::DigitOrZero,
        ]
    );
    assert!(result.negative_section.is_some());
    assert_eq!(
        result.negative_section.as_ref().unwrap().tokens,
        vec![
            FormatToken::LiteralChar('-'),
            FormatToken::DigitIfNeeded,
            FormatToken::ThousandsSeparator,
            FormatToken::DigitIfNeeded,
            FormatToken::DigitIfNeeded,
            FormatToken::DigitOrZero,
            FormatToken::DecimalPoint,
            FormatToken::DigitOrZero,
            FormatToken::DigitOrZero,
        ]
    );

    let result_cond = parse_number_format("[>=1000]#,##0;[<1000]0.0;0;@").unwrap();
    assert!(result_cond.positive_section.condition.is_some());
    assert_eq!(
        result_cond
            .positive_section
            .condition
            .as_ref()
            .unwrap()
            .operator,
        ComparisonOperator::Ge
    );
    assert_eq!(
        result_cond
            .positive_section
            .condition
            .as_ref()
            .unwrap()
            .value,
        1000.0
    );
    assert!(result_cond.negative_section.is_some());
    assert!(
        result_cond
            .negative_section
            .as_ref()
            .unwrap()
            .condition
            .is_some()
    );
    assert_eq!(
        result_cond
            .negative_section
            .as_ref()
            .unwrap()
            .condition
            .as_ref()
            .unwrap()
            .operator,
        ComparisonOperator::Lt
    );

    assert!(result_cond.zero_section.is_some());
    assert!(result_cond.text_section.is_some());
    assert_eq!(
        result_cond.text_section.as_ref().unwrap().tokens,
        vec![FormatToken::TextValue]
    );
}

#[test]
fn test_date_format_month_minute() {
    let result = parse_number_format("mmm d, yyyy h:mm AM/PM").unwrap();
    let tokens = result.positive_section.tokens;

    assert_eq!(tokens[0], FormatToken::MonthAbbr); // mmm

    assert_eq!(
        tokens.len(),
        12,
        "Expected exactly 12 tokens, got {}",
        tokens.len()
    );

    assert_eq!(tokens[1], FormatToken::LiteralChar(' '));
    assert_eq!(tokens[2], FormatToken::DayNum); // d
    assert_eq!(tokens[3], FormatToken::ThousandsSeparator); // ',' 被解析为千位分隔符
    assert_eq!(tokens[4], FormatToken::LiteralChar(' '));
    assert_eq!(tokens[5], FormatToken::YearFourDigit); // yyyy
    assert_eq!(tokens[6], FormatToken::LiteralChar(' '));
    assert_eq!(tokens[7], FormatToken::Hour12Or24); // h
    assert_eq!(tokens[8], FormatToken::LiteralChar(':'));
    assert_eq!(tokens[9], FormatToken::MinuteNumPadded); // mm (resolved from MonthOrMinute2)
    assert_eq!(tokens[10], FormatToken::LiteralChar(' '));
    assert_eq!(tokens[11], FormatToken::AmPm); // AM/PM
}

#[test]
fn test_month_vs_minute() {
    let fmt1 = "hh:mm";
    let res1 = parse_number_format(fmt1).unwrap();
    assert_eq!(
        res1.positive_section.tokens,
        vec![
            FormatToken::Hour12Or24Padded,
            FormatToken::LiteralChar(':'),
            FormatToken::MinuteNumPadded, // mm as minutes
        ]
    );

    let fmt2 = "mm:ss";
    let res2 = parse_number_format(fmt2).unwrap();
    assert_eq!(
        res2.positive_section.tokens,
        vec![
            FormatToken::MinuteNumPadded, // mm as minutes
            FormatToken::LiteralChar(':'),
            FormatToken::SecondNumPadded,
        ]
    );

    let fmt3 = "yyyy-mm-dd";
    let res3 = parse_number_format(fmt3).unwrap();
    assert_eq!(
        res3.positive_section.tokens,
        vec![
            FormatToken::YearFourDigit,
            FormatToken::LiteralChar('-'),
            FormatToken::MonthNumPadded, // mm as month
            FormatToken::LiteralChar('-'),
            FormatToken::DayNumPadded,
        ]
    );

    let fmt4 = "m/d/yy";
    let res4 = parse_number_format(fmt4).unwrap();
    assert_eq!(res4.positive_section.tokens[0], FormatToken::MonthNum); // m as month

    let fmt5 = "h:m";
    let res5 = parse_number_format(fmt5).unwrap();
    assert_eq!(res5.positive_section.tokens[2], FormatToken::MinuteNum); // m as minute
}

#[test]
fn test_quoted_text_with_escapes() {
    let result = parse_number_format("\"hello \\\"world\\\\ \"").unwrap();
    assert_eq!(
        result.positive_section.tokens,
        vec![FormatToken::QuotedText("hello \"world\\ ".to_string()),]
    );
}

#[test]
fn test_text_section_validation() {
    assert!(parse_number_format(";;;@").is_ok());
    assert!(parse_number_format(";;;\"text\"").is_ok());
    assert!(parse_number_format(";;;_ ").is_ok()); // Skip width
    assert!(parse_number_format(";;;* ").is_ok()); // Fill

    assert!(
        parse_number_format(";;;0").is_err(),
        "Should fail: 0 in text section"
    );
    assert!(
        parse_number_format(";;;#").is_err(),
        "Should fail: # in text section"
    );
    assert!(
        parse_number_format(";;;yy").is_err(),
        "Should fail: yy in text section"
    );
    assert!(
        parse_number_format(";;;mmm").is_err(),
        "Should fail: mmm in text section"
    );
}

#[test]
fn test_condition_limits() {
    assert!(parse_number_format("0;-0;0;@").is_ok());
    assert!(parse_number_format("[>0]0;-0;0;@").is_ok());
    assert!(parse_number_format("[>0]0;[<0]-0;0;@").is_ok());
    let res = parse_number_format("[>0]0;[<0]-0;[=0]0;@");
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err(),
        "Format string cannot have more than two conditional sections."
    );
}

#[test]
fn test_empty_sections() {
    let fmt = ";;;";
    let result = parse_number_format(fmt).unwrap();
    assert!(
        result.positive_section.tokens.is_empty() && result.positive_section.condition.is_none()
    );
    assert!(
        result.negative_section.is_some()
            && result.negative_section.as_ref().unwrap().tokens.is_empty()
            && result
                .negative_section
                .as_ref()
                .unwrap()
                .condition
                .is_none()
    );
    assert!(
        result.zero_section.is_some()
            && result.zero_section.as_ref().unwrap().tokens.is_empty()
            && result.zero_section.as_ref().unwrap().condition.is_none()
    );
    assert!(
        result.text_section.is_some()
            && result.text_section.as_ref().unwrap().tokens.is_empty()
            && result.text_section.as_ref().unwrap().condition.is_none()
    );

    let fmt2 = "0.0;;;"; // Empty negative, zero, text
    let result2 = parse_number_format(fmt2).unwrap();
    assert!(!result2.positive_section.tokens.is_empty());
    assert!(
        result2.negative_section.is_some()
            && result2.negative_section.as_ref().unwrap().tokens.is_empty()
    );
}

#[test]
fn exponential_format_tokens() {
    let fmt = "0.00E+00";
    let result = parse_number_format(fmt).unwrap();
    assert_eq!(
        result.positive_section.tokens,
        vec![
            FormatToken::DigitOrZero,
            FormatToken::DecimalPoint,
            FormatToken::DigitOrZero,
            FormatToken::DigitOrZero,
            FormatToken::Exponential(ExponentialNotation::Plus),
            FormatToken::DigitOrZero, // placeholder for exponent
            FormatToken::DigitOrZero, // placeholder for exponent
        ]
    );
}

#[test]
fn test_main() {
    match parse_number_format("#,##0.00;[>=100][Magenta][s].00;\"Zero\";@\" Symbol\"") {
        Ok(parsed_format) => {
            println!("{:#?}", parsed_format);
        }
        Err(e) => {
            eprintln!("Error parsing format: {}", e);
        }
    }
}

#[cfg(test)]
mod formatter_tests {
    use crate::types::{FormatToken, LocaleSettings};
    use crate::{format_number, parse_number_format};

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
}
