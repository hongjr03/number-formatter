use winnow::ascii::float;
use winnow::combinator::{alt, delimited};
use winnow::token::literal;
use winnow::{ModalResult, Parser};

use crate::types::*;

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
