use winnow::ModalResult;
use winnow::Parser;
use winnow::combinator::{alt, opt, peek};
use winnow::error::ContextError;
use winnow::token::literal;

use crate::parser::combinators::parse_condition;
use crate::parser::tokens::*;
use crate::types::*;

/// Parse a sequence of tokens
pub fn parse_section_tokens() -> impl FnMut(&mut &str) -> ModalResult<Vec<FormatToken>> {
    move |input: &mut &str| {
        // This inner logic IS parse_section_tokens_direct
        let mut parts: Vec<FormatToken> = vec![];
        while !input.is_empty() && !lookahead_for_section_separator(input) {
            let date_tokens = alt((
                parse_year_four_digit,
                parse_year_two_digit,
                parse_month_letter,    // mmmmm
                parse_month_full_name, // mmmm
                parse_month_abbr,      // mmm
                // mm and m are handled after more specific time tokens to help ambiguity if possible
                parse_day_full_name, // dddd
                parse_day_abbr,      // ddd
                parse_day_padded,    // dd
                parse_day_single,    // d
            ));

            let time_tokens = alt((
                parse_hour_padded,            // hh
                parse_second_padded,          // ss (try before single m/s)
                parse_month_or_minute_padded, // mm (general, resolved later)
                parse_hour_single,            // h
                parse_second_single,          // s (try before single m)
                parse_month_or_minute_single, // m (general, resolved later)
                parse_am_pm,
                parse_a_p,
                parse_elapsed_hours,
                parse_elapsed_minutes,
                parse_elapsed_seconds,
            ));

            let number_tokens = alt((
                parse_digit_or_zero,
                parse_digit_if_needed,
                parse_digit_or_space,
                parse_decimal_point,
                parse_thousands_separator,
                parse_percentage,
                parse_locale_currency_symbol,
                parse_exponential,
            ));

            let text_special_tokens = alt((
                parse_text_value_token,
                parse_escaped_char_as_literal,
                parse_fill,
                parse_skip_width,
                parse_quoted_text,
                parse_color,
                parse_literal_passthrough, // Should be last in this group
            ));

            let token = alt((date_tokens, time_tokens, number_tokens, text_special_tokens))
                .parse_next(input)?;
            parts.push(token);
        }

        // Use the more detailed ambiguity resolution function
        resolve_month_minute_ambiguity_in_section(&mut parts);

        // Comma transformation logic for datetime sections
        let is_datetime_section = parts.iter().any(FormatToken::is_datetime_placeholder);
        if is_datetime_section {
            for token in parts.iter_mut() {
                if matches!(token, FormatToken::ThousandsSeparator) {
                    *token = FormatToken::LiteralChar(',');
                }
            }
        }
        Ok(parts)
    }
}

/// Parse a single format section
pub fn parse_one_section(
    section_index: usize,
) -> impl FnMut(&mut &str) -> ModalResult<FormatSection> {
    move |input: &mut &str| {
        let is_text_s = section_index == 3;

        let maybe_condition: Option<Condition> = if !is_text_s {
            (opt(parse_condition).parse_next(input))?
        } else {
            None
        };

        // Parse all tokens initially, including all commas as ThousandsSeparator
        let all_tokens: Vec<FormatToken> = (parse_section_tokens().parse_next(input))?;

        // Separate color token if present
        let (color_opt, mut tokens_after_color) = if !all_tokens.is_empty() {
            match &all_tokens[0] {
                FormatToken::Color(color_type) => (
                    Some(color_type.clone()),
                    all_tokens.into_iter().skip(1).collect(),
                ),
                _ => (None, all_tokens),
            }
        } else {
            (None, all_tokens)
        };

        // --- BEGIN: Added logic for fixed denominator and fraction detection ---
        let mut final_tokens: Vec<FormatToken> = Vec::new();
        let mut temp_fixed_denominator: Option<u32> = None;
        let mut temp_has_fraction = false; // Will be set if a non-date slash is found
        let mut temp_has_datetime = false; // Will be set if any datetime token is found
        let mut temp_has_text_format = false; // For @
        let mut temp_num_integer_part_tokens = 0;
        let mut temp_num_fractional_part_tokens = 0;
        let mut in_integer_part = true; // True before a decimal point is encountered (for 0#? counting)
        // or if no decimal point at all.

        let mut tokens_iter = tokens_after_color.into_iter().peekable();
        while let Some(token) = tokens_iter.next() {
            if token.is_datetime_placeholder() {
                temp_has_datetime = true;
            }
            if matches!(token, FormatToken::TextValue) {
                temp_has_text_format = true;
            }

            match token {
                FormatToken::LiteralChar('/') => {
                    if !temp_has_datetime {
                        let mut den_str_chars: Vec<char> = Vec::new();
                        while let Some(FormatToken::LiteralChar(ch)) = tokens_iter.peek() {
                            if ch.is_ascii_digit() {
                                den_str_chars.push(*ch);
                                tokens_iter.next(); // Consume the digit char
                            } else {
                                break; // Not a digit, stop collecting
                            }
                        }

                        if !den_str_chars.is_empty() {
                            let den_str: String = den_str_chars.iter().collect();
                            if let Ok(den_val) = den_str.parse::<u32>() {
                                temp_fixed_denominator = Some(den_val);
                                temp_has_fraction = true;
                                continue;
                            }
                            final_tokens.push(FormatToken::LiteralChar('/'));
                            temp_has_fraction = true;
                        } else {
                            final_tokens.push(FormatToken::LiteralChar('/'));
                            temp_has_fraction = true;
                        }
                    } else {
                        final_tokens.push(FormatToken::LiteralChar('/'));
                    }
                }
                FormatToken::DecimalPoint => {
                    in_integer_part = false;
                    final_tokens.push(token);
                }
                FormatToken::DigitOrZero
                | FormatToken::DigitIfNeeded
                | FormatToken::DigitOrSpace => {
                    if in_integer_part {
                        temp_num_integer_part_tokens += 1;
                    } else {
                        temp_num_fractional_part_tokens += 1;
                    }
                    final_tokens.push(token);
                }
                _ => {
                    final_tokens.push(token);
                }
            }
        }
        tokens_after_color = final_tokens; // Replace with processed tokens
        // --- END: Added logic ---

        let mut num_scaling_commas_val: u8 = 0;

        // Find the index of the last numeric-related token (digit placeholders, decimal, exponential)
        let last_numeric_token_idx = tokens_after_color.iter().rposition(|t| {
            matches!(
                t,
                FormatToken::DigitOrZero
                    | FormatToken::DigitIfNeeded
                    | FormatToken::DigitOrSpace
                    | FormatToken::DecimalPoint
                    | FormatToken::Exponential(_)
            )
        });

        if let Some(last_idx) = last_numeric_token_idx {
            // Numeric part exists. Check for scaling commas after it.
            let mut removal_indices = Vec::new();
            for (i, token) in tokens_after_color.iter().enumerate().skip(last_idx + 1) {
                if matches!(token, FormatToken::ThousandsSeparator) {
                    num_scaling_commas_val += 1;
                    removal_indices.push(i);
                } else {
                    // Stop if a non-comma token is found after the numeric part's trailing commas
                    break;
                }
            }
            // Remove identified scaling commas in reverse order to maintain correct indices
            for i in removal_indices.iter().rev() {
                tokens_after_color.remove(*i);
            }
        } else {
            // No numeric tokens found in the section (e.g., format is just ",,", or ",,TEXT")
            // Only leading commas before any other token type are considered scaling.
            let mut removal_indices = Vec::new();
            for (i, token) in tokens_after_color.iter().enumerate() {
                if matches!(token, FormatToken::ThousandsSeparator) {
                    num_scaling_commas_val += 1;
                    removal_indices.push(i);
                } else {
                    // Stop if any non-comma token is encountered
                    break;
                }
            }
            for i in removal_indices.iter().rev() {
                tokens_after_color.remove(*i);
            }
        }

        Ok(FormatSection {
            condition: maybe_condition,
            tokens: tokens_after_color,
            is_text_section: is_text_s,
            color: color_opt,
            num_scaling_commas: num_scaling_commas_val,
            has_datetime: temp_has_datetime,
            has_text_format: temp_has_text_format,
            has_fraction: temp_has_fraction,
            fixed_denominator: temp_fixed_denominator,
            num_integer_part_tokens: temp_num_integer_part_tokens,
            num_fractional_part_tokens: temp_num_fractional_part_tokens,
        })
    }
}

/// Resolve ambiguity between month and minute tokens (m/mm)
///
/// This function analyzes the context of m/mm tokens to determine whether they represent
/// months or minutes based on adjacent tokens.
pub fn resolve_month_minute_ambiguity_in_section(tokens: &mut Vec<FormatToken>) {
    let mut new_tokens = tokens.clone();
    for i in 0..tokens.len() {
        let (is_m_token, is_single_m) = match tokens[i] {
            FormatToken::MonthOrMinute1 => (true, true),
            FormatToken::MonthOrMinute2 => (true, false),
            _ => (false, false),
        };

        if is_m_token {
            let mut treat_as_minute = false;

            // Rule 1: If preceded by h or hh (e.g., h:mm, hh:mm)
            if i > 0 {
                if matches!(
                    tokens[i - 1],
                    FormatToken::Hour12Or24 | FormatToken::Hour12Or24Padded
                ) {
                    treat_as_minute = true;
                }
                // Rule 2: If preceded by a colon (e.g. :mm)
                // This is often part of h:mm or [h]:mm
                if matches!(tokens[i - 1], FormatToken::LiteralChar(':')) {
                    treat_as_minute = true;
                }
            }

            // Rule 3: If followed by s or ss (e.g., mm:ss)
            if !treat_as_minute && (i + 1 < tokens.len()) {
                if matches!(
                    tokens[i + 1],
                    FormatToken::SecondNum | FormatToken::SecondNumPadded
                ) {
                    treat_as_minute = true;
                }
                // Rule 4: If followed by :s or :ss (e.g., mm:s, mm:ss)
                if i + 2 < tokens.len()
                    && matches!(tokens[i + 1], FormatToken::LiteralChar(':'))
                    && matches!(
                        tokens[i + 2],
                        FormatToken::SecondNum | FormatToken::SecondNumPadded
                    )
                {
                    treat_as_minute = true;
                }
            }

            // Rule 5: If AM/PM token is present anywhere in the section, 'm' or 'mm' are likely minutes.
            // This rule might be too broad if 'mm' is for month in 'yyyy/mm/dd hh:mm AM/PM'.
            // We need to be careful here. Let's prioritize direct neighbor context first.
            if !treat_as_minute {
                let section_has_ampm = tokens
                    .iter()
                    .any(|t| matches!(t, FormatToken::AmPm(_) | FormatToken::AP(_)));
                if section_has_ampm {
                    // If 'm' or 'mm' is NOT directly adjacent to 'd' or 'y' related tokens, and AM/PM is present,
                    // it's more likely a minute. This is a heuristic.
                    let is_near_date_token = (i > 0
                        && matches!(
                            tokens[i - 1],
                            FormatToken::DayNum
                                | FormatToken::DayNumPadded
                                | FormatToken::YearTwoDigit
                                | FormatToken::YearFourDigit
                                | FormatToken::LiteralChar('/')
                                | FormatToken::LiteralChar('-')
                        ))
                        || (i + 1 < tokens.len()
                            && matches!(
                                tokens[i + 1],
                                FormatToken::DayNum
                                    | FormatToken::DayNumPadded
                                    | FormatToken::YearTwoDigit
                                    | FormatToken::YearFourDigit
                                    | FormatToken::LiteralChar('/')
                                    | FormatToken::LiteralChar('-')
                            ));

                    if !is_near_date_token {
                        treat_as_minute = true;
                    }
                }
            }

            if treat_as_minute {
                new_tokens[i] = if is_single_m {
                    FormatToken::MinuteNum
                } else {
                    FormatToken::MinuteNumPadded
                };
            } else {
                new_tokens[i] = if is_single_m {
                    FormatToken::MonthNum
                } else {
                    FormatToken::MonthNumPadded
                };
            }
        }
    }
    *tokens = new_tokens;
}

// Helper parser for a semicolon with the standard ContextError type
fn semicolon_parser<'a>() -> impl Parser<&'a str, &'a str, ContextError<&'a str>> {
    literal(";")
}

// Plausible definition for lookahead_for_section_separator if not found in combinators
// This function checks if the next character is a section separator (e.g., ';')
// without consuming it. Returns true if separator is next, false otherwise.
fn lookahead_for_section_separator(input: &str) -> bool {
    peek(semicolon_parser()).parse_peek(input).is_ok()
    // Adjust the literal if your section separator is different or more complex
}
