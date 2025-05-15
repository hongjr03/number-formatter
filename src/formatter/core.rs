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
    // Datetime and text formatting should take precedence or be handled by specific conditions
    if datetime::section_is_datetime_point_in_time(section) {
        return datetime::format_datetime(original_value_for_sign, section, locale);
    }
    if datetime::section_is_duration(section) {
        return datetime::format_duration(original_value_for_sign, section, locale);
    }
    if section.has_text_format {
        return text::format_text_with_section(&original_value_for_sign.to_string(), section);
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
        let mut result = String::new(); // Shadowing outer result, which is fine here.
        for token in &section.tokens {
            match token {
                FormatToken::LiteralChar(c) => result.push(*c),
                FormatToken::QuotedText(text) => result.push_str(text),
                _ => {} // Other tokens like SkipWidth, Fill if any, are ignored.
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
