use winnow::ascii::float;
use winnow::combinator::{alt, delimited};
use winnow::error::{ContextError, ErrMode, ParserError};
use winnow::token::literal;
use winnow::{ModalResult, Parser};

use crate::parser::tokens::*;
use crate::types::*;

/// Parse a single token from the format string
pub fn parse_single_token(
    is_text_section: bool,
) -> impl FnMut(&mut &str) -> ModalResult<FormatToken> {
    move |input: &mut &str| {
        let original_input_state = *input;

        let group1_datetime_long = alt((
            parse_month_letter,
            parse_month_full_name,
            parse_day_full_name,
            parse_month_abbr,
            parse_day_abbr,
            parse_year_four_digit,
            parse_am_pm,
        ));

        let group2_datetime_twochar_exp = alt((
            parse_exponential,
            parse_year_two_digit,
            parse_month_or_minute_padded,
            parse_day_padded,
            parse_hour_padded,
            parse_second_padded,
            parse_a_p,
        ));

        let group3_datetime_elapsed_single = alt((
            parse_elapsed_hours,
            parse_elapsed_minutes,
            parse_elapsed_seconds,
            parse_month_or_minute_single,
            parse_day_single,
            parse_hour_single,
            parse_second_single,
        ));

        let group4_textual_special = alt((
            parse_quoted_text,
            parse_escaped_char_as_literal,
            parse_fill,
            parse_skip_width,
            parse_color,
        ));

        let group5_number_symbols = alt((
            parse_digit_or_zero,
            parse_digit_if_needed,
            parse_digit_or_space,
            parse_decimal_point,
            parse_thousands_separator,
            parse_percentage,
        ));

        let group6_misc = alt((parse_text_value_token, parse_literal_passthrough));

        let mut parser = alt((
            group1_datetime_long,
            group2_datetime_twochar_exp,
            group3_datetime_elapsed_single,
            group4_textual_special,
            group5_number_symbols,
            group6_misc,
        ));

        match parser.parse_next(input) {
            Ok(token) => {
                if is_text_section && token.is_numeric_or_date() {
                    *input = original_input_state;
                    Err(ErrMode::Backtrack(ContextError::from_input(
                        &original_input_state,
                    )))
                } else {
                    Ok(token)
                }
            }
            Err(e) => Err(e),
        }
    }
}

/// Parse a comparison operator
pub fn parse_comparison_operator_internal(input: &mut &str) -> ModalResult<ComparisonOperator> {
    alt((
        literal("<=").value(ComparisonOperator::Le),
        literal(">=").value(ComparisonOperator::Ge),
        literal("<>").value(ComparisonOperator::Ne),
        literal("=").value(ComparisonOperator::Eq),
        literal("<").value(ComparisonOperator::Lt),
        literal(">").value(ComparisonOperator::Gt),
    ))
    .parse_next(input)
}

/// Parse a condition value (a floating point number)
pub fn parse_condition_value_internal(input: &mut &str) -> ModalResult<f64> {
    float.parse_next(input)
}

/// Parse a condition in the format [operator value]
pub fn parse_condition<'s>(input: &mut &'s str) -> ModalResult<Condition> {
    let core_parser = (
        parse_comparison_operator_internal,
        parse_condition_value_internal,
    )
        .map(|(operator, value)| Condition { operator, value });

    let condition_content = core_parser;

    delimited(
        |i: &mut &'s str| literal("[").parse_next(i),
        condition_content,
        |i: &mut &'s str| literal("]").parse_next(i),
    )
    .parse_next(input)
}
