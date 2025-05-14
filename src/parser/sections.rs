use winnow::combinator::{opt, repeat};
use winnow::{ModalResult, Parser};

use crate::types::*;
use crate::parser::combinators::{parse_condition, parse_single_token};

/// Parse a sequence of tokens
pub fn parse_section_tokens(
    is_text_section: bool,
) -> impl FnMut(&mut &str) -> ModalResult<Vec<FormatToken>> {
    move |input: &mut &str| repeat(0.., parse_single_token(is_text_section)).parse_next(input)
}

/// Parse a single format section
pub fn parse_one_section(section_index: usize) -> impl FnMut(&mut &str) -> ModalResult<FormatSection> {
    move |input: &mut &str| {
        let is_text_s = section_index == 3;

        let maybe_condition: Option<Condition> = if !is_text_s {
            (opt(parse_condition).parse_next(input))?
        } else {
            None
        };

        let tokens: Vec<FormatToken> = (parse_section_tokens(is_text_s).parse_next(input))?;

        Ok(FormatSection {
            condition: maybe_condition,
            tokens,
            is_text_section: is_text_s,
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