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

#[test]
fn test_negative_sign_with_prefix() {
    let locale = LocaleSettings::default();

    // Test case 1: Explicit negative format with prefix
    let format_explicit_neg = parse_number_format("\"USD \"#,##0.00;\"USD \"-#,##0.00").unwrap();
    assert_eq!(
        format_number(-1234.56, &format_explicit_neg, &locale),
        "USD -1,234.56"
    );
    assert_eq!(
        format_number(1234.56, &format_explicit_neg, &locale),
        "USD 1,234.56"
    );
    assert_eq!(
        format_number(-0.50, &format_explicit_neg, &locale),
        "USD -0.50"
    );

    // Test case 2: Implicit negative format handling with prefix
    // Relies on the general sign insertion logic if only positive section is provided
    // and the value is negative.
    let format_implicit_neg = parse_number_format("\"Val: \"#,##0.00").unwrap();
    assert_eq!(
        format_number(-567.89, &format_implicit_neg, &locale),
        "-Val: 567.89" // Assuming default negative is prepending sign
    );
    assert_eq!(
        format_number(567.89, &format_implicit_neg, &locale),
        "Val: 567.89"
    );
    assert_eq!(
        format_number(-0.89, &format_implicit_neg, &locale),
        "-Val: 0.89"
    );
    assert_eq!(
        format_number(0.0, &format_implicit_neg, &locale),
        "Val: 0.00" // Zero handling with prefix
    );

    // Test case 3: Prefix with spaces
    let format_prefix_spaces = parse_number_format("\"  Pref: \"0.0").unwrap();
    assert_eq!(
        format_number(-12.3, &format_prefix_spaces, &locale),
        "-  Pref: 12.3"
    );

    // Test case 4: Negative sign already in prefix literal (should not double sign)
    // This case's behavior depends on how parser/formatter handles literal '-' vs sign.
    // Assuming literal '-' in format string for negative section is the sign.
    let format_literal_minus_in_prefix =
        parse_number_format("\"-Val: \"0.0;\"-Val: \"0.0").unwrap();
    assert_eq!(
        format_number(-12.3, &format_literal_minus_in_prefix, &locale),
        "-Val: 12.3" // or "-Val: -12.3" if it just prepends. This tests if it's smart.
                     // The current core logic should handle the literal '-' from the section
                     // and `sign_printed` should become true.
    );
    // Let's define a positive section and a specific negative one for this.
    // Format: <POSITIVE_FORMAT>;<NEGATIVE_FORMAT_WITH_LITERAL_SIGN>
    let format_neg_section_has_sign =
        parse_number_format("\"Pos: \"0.0;\"NegPrefix -\"0.0").unwrap();
    assert_eq!(
        format_number(-45.6, &format_neg_section_has_sign, &locale),
        "-NegPrefix -45.6"
    );
    let format_literal_minus_in_prefix_2 = parse_number_format("\"-Val: \"0.0").unwrap();
    assert_eq!(
        format_number(-12.3, &format_literal_minus_in_prefix_2, &locale),
        "--Val: 12.3"
    );
}

#[test]
fn test_negative_sign_with_prefix_and_suffix() {
    let locale = LocaleSettings::default();

    // Case 1: Single positive section with literal dash in prefix, negative value
    // Expected: Excel-like prepend of '-' before the section's output.
    let format1 = parse_number_format("\"-Val: \"0.0").unwrap();
    assert_eq!(format_number(-12.3, &format1, &locale), "--Val: 12.3");

    // Case 2: Single positive section, number then text suffix, negative value
    let format2 = parse_number_format("0.0\" USD\"").unwrap();
    assert_eq!(format_number(-12.3, &format2, &locale), "-12.3 USD");

    // Case 3: Defined positive and negative sections, negative section has literal dash prefix
    let format3 = parse_number_format("0.0 \"Pos\";\"-NEG\" 0.0").unwrap();
    assert_eq!(format_number(-12.3, &format3, &locale), "-NEG 12.3");
    assert_eq!(format_number(12.3, &format3, &locale), "12.3 Pos"); // sanity check positive

    // Case 4: Single positive section with literal dash in prefix, zero value
    // Expected: The section is applied as is, no extra sign logic for zero.
    let format4 = parse_number_format("\"-Val: \"0.0").unwrap();
    assert_eq!(format_number(0.0, &format4, &locale), "-Val: 0.0");

    // Case 5: Single positive section with parentheses in prefix/suffix, negative value
    // Excel: -(-Val: 12.3)
    // Our current logic: is_positive_section_fallback_for_negative = true.
    // format_value will produce "(-Val: 12.3)". Then prepends '-'.
    let format5 = parse_number_format("\"(-Val: \"0.0\")\"").unwrap();
    assert_eq!(format_number(-12.3, &format5, &locale), "-(-Val: 12.3)");

    // Case 6: Single positive section, number then text suffix with parentheses, negative value
    // Excel: -12.3 (-)
    let format6 = parse_number_format("0.0\" (-)\"").unwrap();
    assert_eq!(format_number(-12.3, &format6, &locale), "-12.3 (-)");

    // Case 7: Positive section, and Negative section that uses parentheses
    let format7 = parse_number_format("0.0 \"Pos\";(0.0 \"Neg\")").unwrap();
    assert_eq!(format_number(-12.3, &format7, &locale), "(12.3 Neg)");
    assert_eq!(format_number(12.3, &format7, &locale), "12.3 Pos");

    // Case 8: Positive section with literal minus, negative section uses parentheses.
    // This tests that the negative section (with parens) is preferred for negative numbers
    // over applying the single-section-negative-prefix rule to the positive section.
    let format8 = parse_number_format("\"-PosPrefix \"0.0;(0.0 \"NegParen\")").unwrap();
    assert_eq!(format_number(-45.6, &format8, &locale), "(45.6 NegParen)");
    assert_eq!(format_number(45.6, &format8, &locale), "-PosPrefix 45.6");

    // Case 9: Format from user, only positive section "-Val: "0.0
    // This was the original failing case that `is_positive_section_fallback_for_negative` was designed to fix.
    let format9 = parse_number_format("\"-Val: \"0.0").unwrap();
    assert_eq!(format_number(-12.3, &format9, &locale), "--Val: 12.3");
}

#[test]
fn test_special_char_formatting() {
    let locale = LocaleSettings::default();

    // --- Test cases for _ (SkipWidth) ---
    let format_skip1 = parse_number_format("0_0").unwrap();
    assert_eq!(format_number(7.0, &format_skip1, &locale), "7 ");
    assert_eq!(format_number(0.0, &format_skip1, &locale), "0 ");

    let format_skip2 = parse_number_format("\"Result: \"_0").unwrap();
    assert_eq!(format_number(42.0, &format_skip2, &locale), "Result: ");

    // Test for alignment: Positive numbers use _), negative use ()
    // The _ should add one space character.
    let format_skip_align = parse_number_format("0.00_);(0.00)").unwrap();
    assert_eq!(
        format_number(123.45, &format_skip_align, &locale),
        "123.45 "
    );
    assert_eq!(
        format_number(-123.45, &format_skip_align, &locale),
        "(123.45)"
    );

    // --- Test cases for @ (TextValue in numeric context) ---
    let format_at1 = parse_number_format("@").unwrap();
    assert_eq!(format_number(123.45, &format_at1, &locale), "123.45");
    assert_eq!(format_number(0.0, &format_at1, &locale), "0");
    assert_eq!(format_number(-7.0, &format_at1, &locale), "-7");

    let format_at2 = parse_number_format("\"Amt: \"@").unwrap();
    assert_eq!(format_number(123.0, &format_at2, &locale), "123");

    let format_at3 = parse_number_format("@\" units\"").unwrap();
    assert_eq!(format_number(-50.0, &format_at3, &locale), "-50");

    let format_at_section = parse_number_format("0.00;\"NegText: \"@").unwrap();
    assert_eq!(format_number(-67.89, &format_at_section, &locale), "-67.89");
    assert_eq!(format_number(67.89, &format_at_section, &locale), "67.89"); // Positive section

    // --- Test cases for \ (Literal character escape) ---
    // Note: parse_escaped_char_as_literal makes \X -> LiteralChar(X)
    let format_esc_hash = parse_number_format("\\#0").unwrap(); // \# -> LiteralChar('#')
    assert_eq!(format_number(7.0, &format_esc_hash, &locale), "#7");

    let format_esc_underscore = parse_number_format("0\\_0").unwrap(); // \_ -> LiteralChar('_')
    assert_eq!(format_number(8.0, &format_esc_underscore, &locale), "0_8");

    let format_esc_at = parse_number_format("0\\@0").unwrap(); // \@ -> LiteralChar('@')
    assert_eq!(format_number(9.0, &format_esc_at, &locale), "0@9");

    let format_esc_backslash = parse_number_format("0\\\\0").unwrap(); // \\ -> LiteralChar('\')
    assert_eq!(format_number(1.0, &format_esc_backslash, &locale), "0\\1");

    let format_esc_backslash_2 = parse_number_format("0\\\\\\0").unwrap(); // \\ -> LiteralChar('\')
    assert_eq!(format_number(1.0, &format_esc_backslash_2, &locale), "1\\0");

    let format_esc_star = parse_number_format("\\*0").unwrap(); // \* -> LiteralChar('*')
    assert_eq!(format_number(2.0, &format_esc_star, &locale), "*2");
}
