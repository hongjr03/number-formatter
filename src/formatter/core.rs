use crate::formatter::datetime;
use crate::formatter::exponential;
use crate::formatter::text;
use crate::types::{FormatSection, FormatToken, LocaleSettings};

/// Format a numeric value using the specified format section
pub(super) fn format_value(
    original_value_for_sign: f64,
    value_to_format_placeholders: f64,
    section: &FormatSection,
    locale: &LocaleSettings,
    is_positive_section_fallback_for_negative: bool, // True if positive_section is used for a negative original_value
) -> String {
    // Handle empty format string as General
    if section.tokens.is_empty() {
        if original_value_for_sign.is_nan() {
            return "NaN".to_string();
        }
        if original_value_for_sign.is_infinite() {
            return if original_value_for_sign.is_sign_positive() {
                "Infinity"
            } else {
                "-Infinity"
            }
            .to_string();
        }
        if original_value_for_sign == 0.0 {
            return "0".to_string();
        }

        let abs_val = original_value_for_sign.abs();
        let mut s_val;
        let use_scientific = abs_val >= 1E11 || (abs_val < 1E-4 && abs_val != 0.0);

        if use_scientific {
            s_val = format!("{:.6E}", original_value_for_sign);
            if let Some(e_pos) = s_val.find('e').or_else(|| s_val.find('E')) {
                let (mantissa, mut exponent_part) = s_val.split_at(e_pos);
                exponent_part = exponent_part
                    .trim_start_matches('E')
                    .trim_start_matches('e');
                let sign = if exponent_part.starts_with('-') {
                    '-'
                } else {
                    '+'
                };
                let num_str = exponent_part.trim_start_matches(['+', '-']);
                if let Ok(num) = num_str.parse::<i32>() {
                    s_val = format!("{}E{}{:02}", mantissa, sign, num.abs());
                }
            } else {
                s_val = original_value_for_sign.to_string().replace('e', "E"); // Fallback
            }
        } else {
            s_val = original_value_for_sign.to_string();
            let effective_len_check = if original_value_for_sign < 0.0 {
                12
            } else {
                11
            };
            if s_val.contains('.') && s_val.len() > effective_len_check {
                if let Some((full_int_part, frac_part)) = s_val.split_once('.') {
                    let (sign_prefix, numeric_int_part_str) =
                        if let Some(stripped) = full_int_part.strip_prefix('-') {
                            ("-", stripped)
                        } else {
                            ("", full_int_part)
                        };
                    let numeric_int_digits_count = numeric_int_part_str.len();
                    let allowed_frac_digits = 10_usize.saturating_sub(numeric_int_digits_count);
                    if frac_part.len() > allowed_frac_digits {
                        if allowed_frac_digits == 0 {
                            s_val = format!("{}{}", sign_prefix, numeric_int_part_str);
                        } else {
                            s_val = format!(
                                "{}{}.{}",
                                sign_prefix,
                                numeric_int_part_str,
                                &frac_part[..allowed_frac_digits]
                            );
                        }
                    } else if allowed_frac_digits == 0 && !frac_part.is_empty() {
                        s_val = format!("{}{}", sign_prefix, numeric_int_part_str);
                    }
                }
            }
        }
        return s_val;
    }

    // Handle GeneralNumeric first if it's the only token
    if section.tokens.len() == 1 {
        if let FormatToken::GeneralNumeric = section.tokens[0] {
            if original_value_for_sign.is_nan() {
                return "NaN".to_string(); // Consider locale for NaN if needed
            }
            if original_value_for_sign.is_infinite() {
                return if original_value_for_sign.is_sign_positive() {
                    "Infinity"
                } else {
                    "-Infinity"
                }
                .to_string();
            }
            if original_value_for_sign == 0.0 {
                return "0".to_string();
            }

            let abs_val = original_value_for_sign.abs();
            let mut s_val;

            // Determine if scientific notation is needed
            // Excel uses scientific for abs(value) >= 1E11 or (abs(value) < 1E-4 and non-zero)
            // These thresholds are approximate and can depend on context/Excel version.
            let use_scientific = abs_val >= 1E11 || (abs_val < 1E-4 && abs_val != 0.0);

            if use_scientific {
                // Format as X.YYYYYYE+ZZ (approx. 6-7 decimal places for mantissa)
                s_val = format!("{:.6E}", original_value_for_sign);
                // Ensure E is uppercase and exponent is two digits with sign
                if let Some(e_pos) = s_val.find('e').or_else(|| s_val.find('E')) {
                    let (mantissa, mut exponent_part) = s_val.split_at(e_pos);
                    exponent_part = exponent_part
                        .trim_start_matches('E')
                        .trim_start_matches('e');
                    let sign = if exponent_part.starts_with('-') {
                        '-'
                    } else {
                        '+'
                    };
                    let num_str = exponent_part.trim_start_matches(['+', '-']);
                    if let Ok(num) = num_str.parse::<i32>() {
                        s_val = format!("{}E{}{:02}", mantissa, sign, num.abs());
                    } else {
                        // Fallback if exponent parsing fails, just ensure E is uppercase
                        s_val = s_val.replace('e', "E");
                    }
                } else {
                    // Should not happen if format! worked, but as a fallback
                    s_val = original_value_for_sign.to_string().replace('e', "E");
                }
            } else {
                s_val = original_value_for_sign.to_string();
                // For non-scientific, f64::to_string() is generally good.
                // It removes trailing .0 for whole numbers.
                // Targetting around 10 significant digits for General format.

                // If s_val contains a decimal point and its total length is too long for ~10 sig digits.
                let effective_len_check = if original_value_for_sign < 0.0 {
                    12
                } else {
                    11
                };

                if s_val.contains('.') && s_val.len() > effective_len_check {
                    if let Some((full_int_part, frac_part)) = s_val.split_once('.') {
                        let (sign_prefix, numeric_int_part_str) =
                            if let Some(stripped) = full_int_part.strip_prefix('-') {
                                ("-", stripped)
                            } else {
                                ("", full_int_part)
                            };

                        let numeric_int_digits_count = numeric_int_part_str.len();

                        let allowed_frac_digits = 10_usize.saturating_sub(numeric_int_digits_count);

                        if frac_part.len() > allowed_frac_digits {
                            if allowed_frac_digits == 0 {
                                s_val = format!("{}{}", sign_prefix, numeric_int_part_str);
                            } else {
                                s_val = format!(
                                    "{}{}.{}",
                                    sign_prefix,
                                    numeric_int_part_str,
                                    &frac_part[..allowed_frac_digits]
                                );
                            }
                        } else if allowed_frac_digits == 0 && !frac_part.is_empty() {
                            s_val = format!("{}{}", sign_prefix, numeric_int_part_str);
                        }
                    }
                }
            }
            return s_val;
        }
    }

    // Datetime and text formatting should take precedence or be handled by specific conditions
    if datetime::section_is_duration(section) {
        return datetime::format_duration(original_value_for_sign, section, locale);
    }
    if datetime::section_is_datetime_point_in_time(section) {
        return datetime::format_datetime(original_value_for_sign, section, locale);
    }
    if section.has_text_format {
        return text::format_text_with_section(
            &original_value_for_sign.to_string(),
            section,
            locale,
        );
    }

    let analysis = super::fraction::analyze_fraction_pattern(section);
    if analysis.is_fraction_format {
        let mut only_placeholders_and_slash = true;
        let mut seen_slash_in_tokens = false;
        if analysis.has_explicit_slash {
            for token in &section.tokens {
                match token {
                    FormatToken::DigitOrZero
                    | FormatToken::DigitIfNeeded
                    | FormatToken::DigitOrSpace => {}
                    FormatToken::LiteralChar('/') => {
                        seen_slash_in_tokens = true;
                    }
                    FormatToken::LiteralChar(' ') => {}
                    _ => {
                        only_placeholders_and_slash = false;
                        break;
                    }
                }
            }
            if !seen_slash_in_tokens {
                only_placeholders_and_slash = false;
            }
        } else if analysis.fixed_denominator_value.is_some() {
            for token in &section.tokens {
                match token {
                    FormatToken::DigitOrZero
                    | FormatToken::DigitIfNeeded
                    | FormatToken::DigitOrSpace => {}
                    FormatToken::LiteralChar(' ') => {}
                    _ => {
                        only_placeholders_and_slash = false;
                        break;
                    }
                }
            }
        } else {
            only_placeholders_and_slash = false;
        }

        if let Some(fraction_result) = super::fraction::format_number_as_fraction(
            original_value_for_sign,
            value_to_format_placeholders,
            locale,
            &analysis.integer_part_tokens,
            &analysis.numerator_tokens,
            &analysis.denominator_tokens,
            analysis.fixed_denominator_value,
            analysis.has_explicit_slash,
            only_placeholders_and_slash,
        ) {
            return fraction_result;
        }
    }

    // General number formatting logic (non-fraction)

    // Check for sections that are purely literal characters (text output mode)
    let is_text_output_mode = !section.tokens.iter().any(|token| {
        matches!(
            token,
            FormatToken::DigitOrZero
                | FormatToken::DigitIfNeeded
                | FormatToken::DigitOrSpace
                | FormatToken::DecimalPoint
                | FormatToken::Percentage
                | FormatToken::Exponential(_)
                | FormatToken::TextValue // TextValue indicates it's not purely literal in this context
        )
    });

    if is_text_output_mode {
        let mut result = String::new();
        for token in &section.tokens {
            match token {
                FormatToken::LiteralChar(c) => result.push(*c),
                FormatToken::QuotedText(text) => result.push_str(text),
                FormatToken::CurrencySymbolLocaleDefault => {
                    result.push_str(&locale.currency_symbol);
                }
                _ => {}
            }
        }
        return result;
    }

    let abs_value_for_formatting = value_to_format_placeholders.abs();
    let has_percentage = section
        .tokens
        .iter()
        .any(|t| matches!(t, FormatToken::Percentage));

    let mut current_adjusted_value = if has_percentage {
        abs_value_for_formatting * 100.0
    } else {
        abs_value_for_formatting
    };

    if section.num_scaling_commas > 0 {
        for _ in 0..section.num_scaling_commas {
            current_adjusted_value /= 1000.0;
        }
    }

    if let Some(exp_token_idx) = section
        .tokens
        .iter()
        .position(|t| matches!(t, FormatToken::Exponential(_)))
    {
        let value_for_exp =
            if original_value_for_sign < 0.0 && !is_positive_section_fallback_for_negative {
                -current_adjusted_value
            } else {
                current_adjusted_value
            };
        return exponential::format_exponential(value_for_exp, section, exp_token_idx, locale);
    }

    // Call the formatter from the standard_numeric module
    super::standard_numeric::format_standard_numeric_core(
        original_value_for_sign,
        current_adjusted_value,
        section,
        locale,
        is_positive_section_fallback_for_negative,
    )
}
