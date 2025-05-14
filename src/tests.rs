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
    assert_eq!(result.positive_section.tokens.len(), 8);
    assert!(result.negative_section.is_some());
    assert_eq!(result.negative_section.as_ref().unwrap().tokens.len(), 10);

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
    match parse_number_format("#,##0.00;[Red]-#,##0.00;\"Zero\";@\" Symbol\"") {
        Ok(parsed_format) => {
            println!("{:#?}", parsed_format);
        }
        Err(e) => {
            eprintln!("Error parsing format: {}", e);
        }
    }
}
