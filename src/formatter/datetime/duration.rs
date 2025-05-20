use super::utils::{count_fractional_second_digits, format_fractional_seconds};
use crate::types::{FormatSection, FormatToken, LocaleSettings};

/// Helper function to check if a section contains duration-specific tokens
pub fn section_is_duration(section: &FormatSection) -> bool {
    section.tokens.iter().any(|token| {
        matches!(
            token,
            FormatToken::ElapsedHours
                | FormatToken::ElapsedMinutes
                | FormatToken::ElapsedSeconds
                | FormatToken::ElapsedHoursPadded
                | FormatToken::ElapsedMinutesPadded
                | FormatToken::ElapsedSecondsPadded
        )
    })
}

/// Format a duration value according to the format section
pub fn format_duration(
    value: f64, // Excel serial date/time value
    section: &FormatSection,
    _locale: &LocaleSettings, // Placeholder for future use
) -> String {
    if value < 0.0 {
        return format!(
            "ERROR: Negative value ({}) not allowed for duration format.",
            value
        );
    }

    // Convert Excel serial value to total seconds
    let total_seconds_float = value * 86400.0;

    // Count fractional second digits needed
    let num_frac_sec_digits = count_fractional_second_digits(section);

    // Round total seconds according to the desired precision
    let rounded_total_seconds = round_seconds(total_seconds_float, num_frac_sec_digits);

    // Calculate time components
    let time_components = calculate_time_components(rounded_total_seconds);

    // Format duration with tokens
    format_duration_with_tokens(
        section,
        time_components.hours,
        time_components.minutes,
        time_components.minutes_total,
        time_components.seconds,
        time_components.seconds_total,
        time_components.nanos,
    )
}

/// Struct to hold time components for duration formatting
struct TimeComponents {
    hours: i64,         // Hours part (for hh)
    minutes: i64,       // Minutes part (for mm, 0-59)
    minutes_total: i64, // Total minutes (for [m])
    seconds: i64,       // Seconds part (for ss, 0-59)
    seconds_total: i64, // Total seconds (for [s])
    nanos: u32,         // Nanoseconds part
}

/// Round seconds according to the desired precision
fn round_seconds(total_seconds: f64, num_frac_digits: usize) -> f64 {
    if num_frac_digits > 0 {
        let rounding_multiplier = 10f64.powi(num_frac_digits as i32);
        (total_seconds * rounding_multiplier).round() / rounding_multiplier
    } else {
        total_seconds.round() // Round to nearest second if no fractional part in format
    }
}

/// Calculate time components from total seconds
fn calculate_time_components(total_seconds: f64) -> TimeComponents {
    let final_total_seconds_int_part = total_seconds.trunc() as i64;
    // Ensure nanos are calculated from the (potentially) rounded value's fractional part.
    let final_nanos_part = (total_seconds.fract().abs() * 1_000_000_000.0).round() as u32;

    let hours = final_total_seconds_int_part / 3600;
    let minutes_total = final_total_seconds_int_part / 60; // For [m]
    let minutes_part = (final_total_seconds_int_part / 60) % 60;
    let seconds_part = final_total_seconds_int_part % 60;

    TimeComponents {
        hours,
        minutes: minutes_part,
        minutes_total,
        seconds: seconds_part,
        seconds_total: final_total_seconds_int_part,
        nanos: final_nanos_part,
    }
}

/// Format duration with the given format tokens
fn format_duration_with_tokens(
    section: &FormatSection,
    hours: i64,
    minutes: i64,
    minutes_total: i64,
    seconds: i64,
    seconds_total: i64,
    nanos: u32,
) -> String {
    let mut result = String::new();
    let mut tokens_iter = section.tokens.iter().peekable();

    while let Some(token) = tokens_iter.next() {
        match token {
            FormatToken::ElapsedHours => {
                result.push_str(&hours.to_string());
            }
            FormatToken::ElapsedHoursPadded => {
                result.push_str(&format!("{:02}", hours));
            }
            FormatToken::ElapsedMinutes => {
                result.push_str(&minutes_total.to_string());
            }
            FormatToken::ElapsedMinutesPadded => {
                result.push_str(&format!("{:02}", minutes_total));
            }
            FormatToken::ElapsedSeconds => {
                result.push_str(&seconds_total.to_string());
            }
            FormatToken::ElapsedSecondsPadded => {
                result.push_str(&format!("{:02}", seconds_total));
            }
            FormatToken::MonthOrMinute1 | FormatToken::MinuteNum => {
                result.push_str(&minutes.to_string());
            }
            FormatToken::MonthOrMinute2 | FormatToken::MinuteNumPadded => {
                result.push_str(&format!("{:02}", minutes));
            }
            FormatToken::SecondNum | FormatToken::SecondNumPadded => {
                // Format seconds
                if matches!(token, FormatToken::SecondNumPadded) {
                    result.push_str(&format!("{:02}", seconds));
                } else {
                    result.push_str(&seconds.to_string());
                }

                // Handle fractional seconds if any
                handle_fractional_seconds(&mut result, &mut tokens_iter, nanos);
            }
            FormatToken::LiteralChar(c) => result.push(*c),
            FormatToken::QuotedText(text) => result.push_str(text),
            FormatToken::SkipWidth(_) => result.push(' '),
            // Other tokens (Year, Month, Day, Hour12Or24, AmPm, etc.) are generally not expected
            // in pure duration formats. They could be ignored or result in empty output for that part.
            _ => { /* Ignored in duration context for now */ }
        }
    }

    result
}

/// Handle fractional seconds for duration formatting
fn handle_fractional_seconds(
    result: &mut String,
    tokens_iter: &mut std::iter::Peekable<std::slice::Iter<'_, FormatToken>>,
    nanos: u32,
) {
    if let Some(FormatToken::DecimalPoint) = tokens_iter.peek().copied() {
        tokens_iter.next();
        result.push('.');

        let mut current_frac_sec_placeholders = 0;
        let mut count_iter = tokens_iter.clone();
        while let Some(FormatToken::DigitOrZero) = count_iter.peek() {
            count_iter.next();
            current_frac_sec_placeholders += 1;
            if current_frac_sec_placeholders >= 9 {
                break;
            }
        }

        if current_frac_sec_placeholders > 0 {
            let frac_digits = format_fractional_seconds(nanos, current_frac_sec_placeholders);
            result.push_str(&frac_digits);

            // Consume the processed DigitOrZero tokens
            for _ in 0..current_frac_sec_placeholders {
                if tokens_iter
                    .peek()
                    .is_some_and(|t| matches!(t, FormatToken::DigitOrZero))
                {
                    tokens_iter.next();
                } else {
                    break;
                }
            }
        }
    }
}
