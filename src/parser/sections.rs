use winnow::combinator::{opt, repeat};
use winnow::{ModalResult, Parser};

use crate::parser::combinators::{parse_condition, parse_single_token};
use crate::types::*;

/// Parse a sequence of tokens
pub fn parse_section_tokens(
    is_text_section: bool,
) -> impl FnMut(&mut &str) -> ModalResult<Vec<FormatToken>> {
    move |input: &mut &str| repeat(0.., parse_single_token(is_text_section)).parse_next(input)
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
        let all_tokens: Vec<FormatToken> = (parse_section_tokens(is_text_s).parse_next(input))?;

        // Separate color token if present
        let (color_opt, mut tokens_after_color) = if !all_tokens.is_empty() {
            match &all_tokens[0] {
                FormatToken::Color(color_type) => {
                    (Some(color_type.clone()), all_tokens.into_iter().skip(1).collect())
                }
                _ => (None, all_tokens),
            }
        } else {
            (None, all_tokens)
        };

        let mut num_scaling_commas_val: u8 = 0;

        // Find the index of the last numeric-related token (digit placeholders, decimal, exponential)
        let last_numeric_token_idx = tokens_after_color.iter().rposition(|t| {
            matches!(t, FormatToken::DigitOrZero |
                        FormatToken::DigitIfNeeded |
                        FormatToken::DigitOrSpace |
                        FormatToken::DecimalPoint |
                        FormatToken::Exponential(_))
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
            if i > 0 {
                if matches!(
                    tokens[i - 1],
                    FormatToken::Hour12Or24 | FormatToken::Hour12Or24Padded
                ) {
                    treat_as_minute = true;
                }

                if matches!(tokens[i - 1], FormatToken::LiteralChar(':')) {
                    treat_as_minute = true;
                }
            }
            if !treat_as_minute && (i + 1 < tokens.len()) {
                if matches!(
                    tokens[i + 1],
                    FormatToken::SecondNum | FormatToken::SecondNumPadded
                ) {
                    treat_as_minute = true;
                }

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
