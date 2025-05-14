use winnow::ascii::Caseless;
use winnow::combinator::{alt, delimited, preceded, repeat};
use winnow::error::ErrMode;
use winnow::token::{any, literal, none_of, one_of};
use winnow::{ModalResult, Parser};

use crate::types::*;

// Year related parsers
pub fn parse_year_four_digit(input: &mut &str) -> ModalResult<FormatToken> {
    literal(Caseless("yyyy"))
        .value(FormatToken::YearFourDigit)
        .parse_next(input)
        .map_err(ErrMode::Backtrack)
}

pub fn parse_year_two_digit(input: &mut &str) -> ModalResult<FormatToken> {
    literal(Caseless("yy"))
        .value(FormatToken::YearTwoDigit)
        .parse_next(input)
        .map_err(ErrMode::Backtrack)
}

// Month related parsers
pub fn parse_month_letter(input: &mut &str) -> ModalResult<FormatToken> {
    literal(Caseless("mmmmm"))
        .value(FormatToken::MonthLetter)
        .parse_next(input)
        .map_err(ErrMode::Backtrack)
}

pub fn parse_month_full_name(input: &mut &str) -> ModalResult<FormatToken> {
    literal(Caseless("mmmm"))
        .value(FormatToken::MonthFullName)
        .parse_next(input)
        .map_err(ErrMode::Backtrack)
}

pub fn parse_month_abbr(input: &mut &str) -> ModalResult<FormatToken> {
    literal(Caseless("mmm"))
        .value(FormatToken::MonthAbbr)
        .parse_next(input)
        .map_err(ErrMode::Backtrack)
}

pub fn parse_month_or_minute_padded(input: &mut &str) -> ModalResult<FormatToken> {
    literal(Caseless("mm"))
        .value(FormatToken::MonthOrMinute2)
        .parse_next(input)
        .map_err(ErrMode::Backtrack)
}

pub fn parse_month_or_minute_single(input: &mut &str) -> ModalResult<FormatToken> {
    literal(Caseless("m"))
        .value(FormatToken::MonthOrMinute1)
        .parse_next(input)
        .map_err(ErrMode::Backtrack)
}

// Day related parsers
pub fn parse_day_full_name(input: &mut &str) -> ModalResult<FormatToken> {
    literal(Caseless("dddd"))
        .value(FormatToken::WeekdayFullName)
        .parse_next(input)
        .map_err(ErrMode::Backtrack)
}

pub fn parse_day_abbr(input: &mut &str) -> ModalResult<FormatToken> {
    literal(Caseless("ddd"))
        .value(FormatToken::WeekdayAbbr)
        .parse_next(input)
        .map_err(ErrMode::Backtrack)
}

pub fn parse_day_padded(input: &mut &str) -> ModalResult<FormatToken> {
    literal(Caseless("dd"))
        .value(FormatToken::DayNumPadded)
        .parse_next(input)
        .map_err(ErrMode::Backtrack)
}

pub fn parse_day_single(input: &mut &str) -> ModalResult<FormatToken> {
    literal(Caseless("d"))
        .value(FormatToken::DayNum)
        .parse_next(input)
        .map_err(ErrMode::Backtrack)
}

// Time related parsers
pub fn parse_hour_padded(input: &mut &str) -> ModalResult<FormatToken> {
    literal(Caseless("hh"))
        .value(FormatToken::Hour12Or24Padded)
        .parse_next(input)
        .map_err(ErrMode::Backtrack)
}

pub fn parse_hour_single(input: &mut &str) -> ModalResult<FormatToken> {
    literal(Caseless("h"))
        .value(FormatToken::Hour12Or24)
        .parse_next(input)
        .map_err(ErrMode::Backtrack)
}

pub fn parse_second_padded(input: &mut &str) -> ModalResult<FormatToken> {
    literal(Caseless("ss"))
        .value(FormatToken::SecondNumPadded)
        .parse_next(input)
        .map_err(ErrMode::Backtrack)
}

pub fn parse_second_single(input: &mut &str) -> ModalResult<FormatToken> {
    literal(Caseless("s"))
        .value(FormatToken::SecondNum)
        .parse_next(input)
        .map_err(ErrMode::Backtrack)
}

pub fn parse_am_pm(input: &mut &str) -> ModalResult<FormatToken> {
    literal(Caseless("AM/PM"))
        .value(FormatToken::AmPm)
        .parse_next(input)
        .map_err(ErrMode::Backtrack)
}

pub fn parse_a_p(input: &mut &str) -> ModalResult<FormatToken> {
    literal(Caseless("A/P"))
        .value(FormatToken::AP)
        .parse_next(input)
        .map_err(ErrMode::Backtrack)
}

// Elapsed time parsers
pub fn parse_elapsed_hours(input: &mut &str) -> ModalResult<FormatToken> {
    delimited(
        literal(Caseless("[")),
        literal(Caseless("h")),
        literal(Caseless("]")),
    )
    .value(FormatToken::ElapsedHours)
    .parse_next(input)
    .map_err(ErrMode::Backtrack)
}

pub fn parse_elapsed_minutes(input: &mut &str) -> ModalResult<FormatToken> {
    delimited(
        literal(Caseless("[")),
        literal(Caseless("m")),
        literal(Caseless("]")),
    )
    .value(FormatToken::ElapsedMinutes)
    .parse_next(input)
    .map_err(ErrMode::Backtrack)
}

pub fn parse_elapsed_seconds(input: &mut &str) -> ModalResult<FormatToken> {
    delimited(
        literal(Caseless("[")),
        literal(Caseless("s")),
        literal(Caseless("]")),
    )
    .value(FormatToken::ElapsedSeconds)
    .parse_next(input)
    .map_err(ErrMode::Backtrack)
}

// Number format parsers
pub fn parse_digit_or_zero(input: &mut &str) -> ModalResult<FormatToken> {
    literal("0")
        .value(FormatToken::DigitOrZero)
        .parse_next(input)
        .map_err(ErrMode::Backtrack)
}

pub fn parse_digit_if_needed(input: &mut &str) -> ModalResult<FormatToken> {
    literal("#")
        .value(FormatToken::DigitIfNeeded)
        .parse_next(input)
        .map_err(ErrMode::Backtrack)
}

pub fn parse_digit_or_space(input: &mut &str) -> ModalResult<FormatToken> {
    literal("?")
        .value(FormatToken::DigitOrSpace)
        .parse_next(input)
        .map_err(ErrMode::Backtrack)
}

pub fn parse_decimal_point(input: &mut &str) -> ModalResult<FormatToken> {
    literal(".")
        .value(FormatToken::DecimalPoint)
        .parse_next(input)
        .map_err(ErrMode::Backtrack)
}

pub fn parse_thousands_separator(input: &mut &str) -> ModalResult<FormatToken> {
    literal(",")
        .value(FormatToken::ThousandsSeparator)
        .parse_next(input)
        .map_err(ErrMode::Backtrack)
}

pub fn parse_percentage(input: &mut &str) -> ModalResult<FormatToken> {
    literal("%")
        .value(FormatToken::Percentage)
        .parse_next(input)
        .map_err(ErrMode::Backtrack)
}

pub fn parse_exponential(input: &mut &str) -> ModalResult<FormatToken> {
    alt((
        literal(Caseless("E+")).value(FormatToken::Exponential(ExponentialNotation::Plus)),
        literal(Caseless("E-")).value(FormatToken::Exponential(ExponentialNotation::Minus)),
    ))
    .parse_next(input)
    .map_err(ErrMode::Backtrack)
}

// Text and special character parsers
pub fn parse_text_value_token(input: &mut &str) -> ModalResult<FormatToken> {
    literal("@")
        .value(FormatToken::TextValue)
        .parse_next(input)
        .map_err(ErrMode::Backtrack)
}

pub fn parse_escaped_char_as_literal(input: &mut &str) -> ModalResult<FormatToken> {
    preceded('\\', any)
        .map(FormatToken::LiteralChar)
        .parse_next(input)
        .map_err(ErrMode::Backtrack)
}

pub fn parse_literal_passthrough(input: &mut &str) -> ModalResult<FormatToken> {
    one_of(['$', '-', '+', '/', '(', ')', ' ', ':'])
        .map(FormatToken::LiteralChar)
        .parse_next(input)
        .map_err(ErrMode::Backtrack)
}

pub fn parse_fill(input: &mut &str) -> ModalResult<FormatToken> {
    preceded('*', any)
        .map(FormatToken::Fill)
        .parse_next(input)
        .map_err(ErrMode::Backtrack)
}

pub fn parse_skip_width(input: &mut &str) -> ModalResult<FormatToken> {
    preceded('_', any)
        .map(FormatToken::SkipWidth)
        .parse_next(input)
        .map_err(ErrMode::Backtrack)
}

pub fn parse_quoted_text(input: &mut &str) -> ModalResult<FormatToken> {
    let content_parser = repeat(0.., alt((preceded('\\', any), none_of(['"']))))
        .map(|chars: Vec<char>| chars.into_iter().collect::<String>());

    delimited('"', content_parser, '"')
        .map(FormatToken::QuotedText)
        .parse_next(input)
        .map_err(ErrMode::Backtrack)
}

// Color parsers
pub fn parse_color(input: &mut &str) -> ModalResult<FormatToken> {
    let red = literal(Caseless("[Red]")).value(FormatToken::Color(ColorType::Red));
    let green = literal(Caseless("[Green]")).value(FormatToken::Color(ColorType::Green));
    let blue = literal(Caseless("[Blue]")).value(FormatToken::Color(ColorType::Blue));
    let magenta = literal(Caseless("[Magenta]")).value(FormatToken::Color(ColorType::Magenta));
    let cyan = literal(Caseless("[Cyan]")).value(FormatToken::Color(ColorType::Cyan));
    let yellow = literal(Caseless("[Yellow]")).value(FormatToken::Color(ColorType::Yellow));
    let black = literal(Caseless("[Black]")).value(FormatToken::Color(ColorType::Black));
    let white = literal(Caseless("[White]")).value(FormatToken::Color(ColorType::White));

    alt((red, green, blue, magenta, cyan, yellow, black, white))
        .parse_next(input)
        .map_err(ErrMode::Backtrack)
}
