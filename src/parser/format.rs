use winnow::Parser;
use winnow::error::ErrMode;
use winnow::token::literal;

use crate::parser::sections::{parse_one_section, resolve_month_minute_ambiguity_in_section};
use crate::types::*;

/// Parse a number format string
///
/// This is the main public API entry point of this module. It accepts a format string and returns the parsed NumberFormat structure.
///
/// # Arguments
/// * `input_str` - The format string to parse
///
/// # Returns
/// * `Result<NumberFormat, String>` - The parsing result, or an error message
///
/// # Examples
/// ```
/// use number_format::parser::parse_number_format;
///
/// let result = parse_number_format("0.00").unwrap();
/// ```
pub fn parse_number_format(input_str: &str) -> Result<NumberFormat, String> {
    // Handle "General" format as a special case
    // If the input string CONTAINS "general" (case-insensitive), treat it as General format.
    if input_str.to_lowercase().contains("general") {
        let general_section = FormatSection {
            tokens: vec![FormatToken::GeneralNumeric],
            color: None,
            condition: None,
            is_text_section: false,
            num_scaling_commas: 0,
            has_datetime: false,
            has_text_format: false,
            has_fraction: false,
            fixed_denominator: None,
            num_integer_part_tokens: 0,
            num_fractional_part_tokens: 0,
        };

        let general_text_section = FormatSection {
            tokens: vec![FormatToken::TextValue], // @ token
            color: None,
            condition: None,
            is_text_section: true,
            num_scaling_commas: 0,
            has_datetime: false,
            has_text_format: true,
            has_fraction: false,
            fixed_denominator: None,
            num_integer_part_tokens: 0,
            num_fractional_part_tokens: 0,
        };

        return Ok(NumberFormat {
            positive_section: general_section,
            negative_section: None,
            zero_section: None,
            text_section: Some(general_text_section),
        });
    }

    let mut input = input_str;

    let make_err_msg = |e, remaining: &str| -> String {
        format!("Parse error: {e:?} at remaining input '{remaining}'")
    };

    let mut positive_section = parse_one_section(0)
        .parse_next(&mut input)
        .map_err(|e| make_err_msg(e, input))?;

    let mut negative_section = None;
    if input.starts_with(';') {
        literal(";")
            .parse_next(&mut input)
            .map_err(ErrMode::Backtrack)
            .map_err(|e| make_err_msg(e, input))?;
        negative_section = Some(
            parse_one_section(1)
                .parse_next(&mut input)
                .map_err(|e| make_err_msg(e, input))?,
        );
    }

    let mut zero_section = None;
    if input.starts_with(';') {
        literal(";")
            .parse_next(&mut input)
            .map_err(ErrMode::Backtrack)
            .map_err(|e| make_err_msg(e, input))?;
        zero_section = Some(
            parse_one_section(2)
                .parse_next(&mut input)
                .map_err(|e| make_err_msg(e, input))?,
        );
    }

    let mut text_section = None;
    if input.starts_with(';') {
        literal(";")
            .parse_next(&mut input)
            .map_err(ErrMode::Backtrack)
            .map_err(|e| make_err_msg(e, input))?;
        text_section = Some(
            parse_one_section(3)
                .parse_next(&mut input)
                .map_err(|e| make_err_msg(e, input))?,
        );
    }

    if !input.is_empty() {
        return Err(format!(
            "Too many sections or trailing characters: '{input}'"
        ));
    }

    // Resolve month/minute ambiguity in all sections
    resolve_month_minute_ambiguity_in_section(&mut positive_section.tokens);
    if let Some(ref mut section) = negative_section {
        resolve_month_minute_ambiguity_in_section(&mut section.tokens);
    }
    if let Some(ref mut section) = zero_section {
        resolve_month_minute_ambiguity_in_section(&mut section.tokens);
    }
    if let Some(ref mut section) = text_section {
        resolve_month_minute_ambiguity_in_section(&mut section.tokens);
        if section.condition.is_some() {
            return Err("Text section (4th) must not have a condition.".to_string());
        }
    }

    // Validate condition constraints
    let mut condition_count = 0;
    if positive_section.condition.is_some() {
        condition_count += 1;
    }
    if negative_section
        .as_ref()
        .is_some_and(|s| s.condition.is_some())
    {
        condition_count += 1;
    }
    if zero_section.as_ref().is_some_and(|s| s.condition.is_some()) {
        condition_count += 1;
    }

    if condition_count > 2 {
        return Err("Format string cannot have more than two conditional sections.".to_string());
    }

    // Validate text section
    if let Some(ref section) = text_section {
        for token in &section.tokens {
            if token.is_numeric_or_date() {
                return Err(format!(
                    "Text section (4th) contains a numeric or date symbol: {token:?}"
                ));
            }
        }
    }

    Ok(NumberFormat {
        positive_section,
        negative_section,
        zero_section,
        text_section,
    })
}
