use winnow::ascii::Caseless;
use winnow::combinator::{alt, delimited, preceded, repeat};
use winnow::error::{ContextError, ErrMode};
use winnow::token::{any, literal, none_of, one_of};
use winnow::{ModalResult, Parser};

use crate::types::*;

pub fn parse_year_four_digit(input: &mut &str) -> ModalResult<FormatToken> {
    repeat::<_, _, (), ContextError, _>(3.., one_of(('y', 'Y')).map(|_| ()))
        .value(FormatToken::YearFourDigit)
        .parse_next(input)
        .map_err(ErrMode::Backtrack)
}

pub fn parse_year_two_digit(input: &mut &str) -> ModalResult<FormatToken> {
    repeat::<_, _, (), ContextError, _>(1..3, one_of(('y', 'Y')).map(|_| ()))
        .value(FormatToken::YearTwoDigit)
        .parse_next(input)
        .map_err(ErrMode::Backtrack)
}

pub fn parse_month_letter(input: &mut &str) -> ModalResult<FormatToken> {
    literal(Caseless("mmmmm"))
        .value(FormatToken::MonthLetter)
        .parse_next(input)
        .map_err(ErrMode::Backtrack)
}

pub fn parse_month_full_name_long(input: &mut &str) -> ModalResult<FormatToken> {
    repeat::<_, _, (), ContextError, _>(6.., one_of(('m', 'M')).map(|_| ()))
        .value(FormatToken::MonthFullName)
        .parse_next(input)
        .map_err(ErrMode::Backtrack)
}

pub fn parse_month_full_name(input: &mut &str) -> ModalResult<FormatToken> {
    // mmmm or mmm.. (n>5)
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

pub fn parse_day_full_name(input: &mut &str) -> ModalResult<FormatToken> {
    alt((
        repeat::<_, _, (), ContextError, _>(4.., one_of(('d', 'D')).map(|_| ())),
        repeat::<_, _, (), ContextError, _>(4.., one_of(('a', 'A')).map(|_| ())),
    ))
    .value(FormatToken::WeekdayFullName)
    .parse_next(input)
    .map_err(ErrMode::Backtrack)
}

pub fn parse_day_abbr(input: &mut &str) -> ModalResult<FormatToken> {
    alt((literal(Caseless("ddd")), literal(Caseless("aaa"))))
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
    repeat::<_, _, (), ContextError, _>(2.., one_of(('h', 'H')).map(|_| ()))
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
    repeat::<_, _, (), ContextError, _>(2.., one_of(('s', 'S')).map(|_| ()))
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
    alt((
        literal("AM/PM").value(FormatToken::AmPm(AmPmStyle::UpperCase)),
        literal("am/pm").value(FormatToken::AmPm(AmPmStyle::LowerCase)),
    ))
    .parse_next(input)
    .map_err(ErrMode::Backtrack)
}

pub fn parse_a_p(input: &mut &str) -> ModalResult<FormatToken> {
    alt((
        literal("A/P").value(FormatToken::AP(AmPmStyle::UpperCase)),
        literal("a/p").value(FormatToken::AP(AmPmStyle::LowerCase)),
    ))
    .parse_next(input)
    .map_err(ErrMode::Backtrack)
}

// Elapsed time parsers
pub fn parse_elapsed_hours(input: &mut &str) -> ModalResult<FormatToken> {
    delimited(literal("["), literal(Caseless("h")), literal("]"))
        .value(FormatToken::ElapsedHours)
        .parse_next(input)
        .map_err(ErrMode::Backtrack)
}

pub fn parse_elapsed_minutes(input: &mut &str) -> ModalResult<FormatToken> {
    delimited(literal("["), literal(Caseless("m")), literal("]"))
        .value(FormatToken::ElapsedMinutes)
        .parse_next(input)
        .map_err(ErrMode::Backtrack)
}

pub fn parse_elapsed_seconds(input: &mut &str) -> ModalResult<FormatToken> {
    delimited(literal("["), literal(Caseless("s")), literal("]"))
        .value(FormatToken::ElapsedSeconds)
        .parse_next(input)
        .map_err(ErrMode::Backtrack)
}

pub fn parse_elapsed_hours_padded(input: &mut &str) -> ModalResult<FormatToken> {
    delimited(literal("["), literal(Caseless("hh")), literal("]"))
        .value(FormatToken::ElapsedHoursPadded)
        .parse_next(input)
        .map_err(ErrMode::Backtrack)
}

pub fn parse_elapsed_minutes_padded(input: &mut &str) -> ModalResult<FormatToken> {
    delimited(literal("["), literal(Caseless("mm")), literal("]"))
        .value(FormatToken::ElapsedMinutesPadded)
        .parse_next(input)
        .map_err(ErrMode::Backtrack)
}

pub fn parse_elapsed_seconds_padded(input: &mut &str) -> ModalResult<FormatToken> {
    delimited(literal("["), literal(Caseless("ss")), literal("]"))
        .value(FormatToken::ElapsedSecondsPadded)
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

pub fn parse_literal_percentage_sign(input: &mut &str) -> ModalResult<FormatToken> {
    literal("%%")
        .value(FormatToken::LiteralChar('%'))
        .parse_next(input)
        .map_err(ErrMode::Backtrack)
}

pub fn parse_percentage(input: &mut &str) -> ModalResult<FormatToken> {
    literal('%')
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
    one_of([
        '$', '+', '(', ':', '^', '\'', '{', '<', '=', '-', '/', ')', '!', '&', '~', '}', '>', ' ',
    ])
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

pub fn parse_locale_currency_symbol(input: &mut &str) -> ModalResult<FormatToken> {
    alt((
        literal('¤').value(FormatToken::CurrencySymbolLocaleDefault),
        parse_excel_locale_currency_format,
    ))
    .parse_next(input)
}

/// Parse Excel-style locale currency format like [$-409] or [$-zh-TW] or [$US-409]
pub fn parse_excel_locale_currency_format(input: &mut &str) -> ModalResult<FormatToken> {
    // Parse the opening sequence [$
    literal("[$").parse_next(input)?;

    // Check for currency prefix like US before the - sign
    let mut currency_prefix = String::new();
    let original_input = *input;

    // Read characters until we hit a dash
    while !input.is_empty() && !input.starts_with('-') {
        let c = input.chars().next().unwrap();
        currency_prefix.push(c);
        *input = &input[c.len_utf8()..];
    }

    // Parse the dash
    if literal::<_, _, ContextError>("-")
        .parse_next(input)
        .is_err()
    {
        // No dash found, restore input and clear prefix
        *input = original_input;
        currency_prefix.clear();
    }

    // Now parse the locale code which can be:
    // 1. A numeric code like 409
    // 2. A hex code like 1C or 5E
    // 3. A language code like zh-TW
    let mut locale_code = String::new();

    // Consume characters until we hit the closing bracket
    while !input.is_empty() && !input.starts_with(']') {
        // Take one character at a time
        let c = input.chars().next().unwrap();
        locale_code.push(c);
        *input = &input[c.len_utf8()..];
    }

    // Parse the closing bracket
    literal("]").parse_next(input)?;

    // Generate the full locale code for later reference
    let full_code = format!("[$-{}]", locale_code);

    // Return appropriate token based on whether there's a currency prefix
    if !currency_prefix.is_empty() {
        // Include both the prefix and the locale code for complete formatting
        Ok(FormatToken::CurrencySymbolLocalePrefixed(format!(
            "{}:{}",
            currency_prefix, full_code
        )))
    } else {
        // Just store the locale code for using the default currency symbol of that locale
        Ok(FormatToken::CurrencySymbolLocaleDefault)
    }
}
