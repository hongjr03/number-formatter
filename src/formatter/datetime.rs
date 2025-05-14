use crate::types::{AmPmStyle, FormatSection, FormatToken, LocaleSettings};
use chrono::{Datelike, NaiveDate, NaiveDateTime, NaiveTime, Timelike};

// TODO: Determine the correct Excel epoch (1900-01-00 or 1904-01-01)
// Excel's 1900 epoch has a bug where it considers 1900 a leap year.
// For simplicity, let's assume a base and handle f64 conversion carefully.
// const EXCEL_EPOCH_DATE: NaiveDate = NaiveDate::from_ymd_opt(1899, 12, 30).unwrap(); // Common base for f64 to date

// Helper function to convert f64 Excel date to NaiveDateTime
// Takes into account Excel's leap year bug (1900-02-29 is valid)
fn convert_f64_to_datetime(value: f64) -> Option<NaiveDateTime> {
    if value < 0.0 {
        // Excel serial dates are typically non-negative.
        // Some interpretations map negative numbers to dates before 1900-01-01,
        // but for formatting, it's often an error or undefined.
        return None;
    }

    let excel_serial_day_part = value.trunc() as i64;
    let time_fraction = value.fract();

    // Date part calculation
    let date_part = if excel_serial_day_part == 0 {
        // Serial 0 is conventionally 1899-12-31
        NaiveDate::from_ymd_opt(1899, 12, 31)?
    } else {
        // For other serial numbers (including 60, which will be handled by format_datetime directly for 1900-02-29)
        // Base date for serial 1 is 1900-01-01.
        // Days to add from 1900-01-01:
        // - For serials 1-59, it's (serial - 1) days.
        // - For serials >60, it's (serial - 2) days to account for the phantom 1900-02-29.
        let days_offset_from_1900_01_01 = if excel_serial_day_part > 60 {
            excel_serial_day_part - 2
        } else {
            // This covers 1 to 59 (since 0 and 60 are special-cased)
            excel_serial_day_part - 1
        };
        NaiveDate::from_ymd_opt(1900, 1, 1)?
            .checked_add_signed(chrono::Duration::days(days_offset_from_1900_01_01))?
    };

    // Time part calculation
    // Ensure time_fraction is positive for calculation.
    // value >= 0 implies time_fraction >= 0.
    let mut total_seconds_precise = time_fraction * 86400.0;

    let mut current_date_part = date_part;

    // Handle rollover if time fraction causes seconds to be >= 86400.0
    if total_seconds_precise >= 86400.0 {
        let extra_days = (total_seconds_precise / 86400.0).trunc() as i64;
        if extra_days > 0 {
            current_date_part =
                current_date_part.checked_add_signed(chrono::Duration::days(extra_days))?;
            total_seconds_precise -= (extra_days as f64) * 86400.0;
            // Clamp to prevent issues if it's still somehow >= 86400 after subtracting full days
            if total_seconds_precise >= 86400.0 {
                total_seconds_precise = 86400.0 - 1e-9; // Just under a full day
            }
        }
    }
    // Ensure total_seconds_precise is strictly less than 86400 for h/m/s/ns calculation
    // This handles cases like exactly 86400.0 after potential rollover subtraction,
    // or if initial time_fraction was 1.0 (value was an integer).
    if total_seconds_precise >= 86400.0 {
        total_seconds_precise = 0.0; // Should have rolled over to next day
        // If it was exactly 1.0 and rolled over, date is already correct.
        // If it was slightly more and rolled over, date and remaining seconds are correct.
        // If input value was an integer, time_fraction is 0, total_seconds_precise is 0.
    }

    let hours = (total_seconds_precise / 3600.0).trunc() as u32;
    let minutes = ((total_seconds_precise % 3600.0) / 60.0).trunc() as u32;
    let seconds = (total_seconds_precise % 60.0).trunc() as u32;

    // Nanoseconds part: (total_seconds_precise.fract() * 1_000_000_000.0).round() as u32
    // or more robustly from the original time_fraction to minimize intermediate floating point errors
    // let nanoseconds = ( (time_fraction * 86400.0) % 1.0 * 1_000_000_000.0).round() as u32;
    // No, use total_seconds_precise which has been adjusted for day rollovers
    let nanoseconds = ((total_seconds_precise % 1.0) * 1_000_000_000.0).round() as u32;

    // Clamp nanoseconds to max value for NaiveTime::from_hms_nano_opt
    let clamped_nanos = nanoseconds.min(999_999_999);

    let time_part = NaiveTime::from_hms_nano_opt(hours, minutes, seconds, clamped_nanos)?;

    Some(NaiveDateTime::new(current_date_part, time_part))
}

pub(super) fn format_datetime(
    value: f64,
    section: &FormatSection,
    locale: &LocaleSettings,
) -> String {
    // Handle Excel's 1900-02-29 (serial 60) directly
    if value.trunc() == 60.0 && value.fract() == 0.0 {
        let mut special_result = String::new();
        for token in &section.tokens {
            match token {
                FormatToken::YearTwoDigit => special_result.push_str("00"),
                FormatToken::YearFourDigit => special_result.push_str("1900"),
                FormatToken::MonthNum => special_result.push('2'),
                FormatToken::MonthNumPadded => special_result.push_str("02"),
                FormatToken::MonthAbbr => special_result.push_str(&locale.short_month_names[1]), // Index 1 for February
                FormatToken::MonthFullName => special_result.push_str(&locale.month_names[1]), // Index 1 for February
                FormatToken::MonthLetter => special_result.push('F'), // February
                FormatToken::DayNum => special_result.push_str("29"),
                FormatToken::DayNumPadded => special_result.push_str("29"),
                FormatToken::WeekdayAbbr => special_result.push_str(&locale.short_day_names[3]), // Wednesday (Excel)
                FormatToken::WeekdayFullName => special_result.push_str(&locale.day_names[3]), // Wednesday (Excel)
                FormatToken::LiteralChar(c) => special_result.push(*c),
                FormatToken::QuotedText(text) => special_result.push_str(text),
                FormatToken::SkipWidth(_) => special_result.push(' '),
                _ => {}
            }
        }
        return special_result;
    }

    let Some(dt) = convert_f64_to_datetime(value) else {
        return format!("INVALID_DATE_SERIAL: {}", value);
    };

    let mut result = String::new(); // This is the main result string for non-serial-60 dates

    let has_ampm = section
        .tokens
        .iter()
        .any(|t| matches!(t, FormatToken::AmPm(_) | FormatToken::AP(_)));

    let mut tokens_iter = section.tokens.iter().peekable();
    while let Some(token) = tokens_iter.next() {
        match token {
            // Date tokens
            FormatToken::YearTwoDigit => {
                result.push_str(&format!("{:02}", dt.year() % 100));
            }
            FormatToken::YearFourDigit => {
                result.push_str(&dt.year().to_string());
            }
            FormatToken::MonthNum => {
                result.push_str(&dt.month().to_string());
            }
            FormatToken::MonthNumPadded => {
                result.push_str(&format!("{:02}", dt.month()));
            }
            FormatToken::MonthAbbr => {
                result.push_str(&locale.short_month_names[dt.month0() as usize]);
            }
            FormatToken::MonthFullName => {
                result.push_str(&locale.month_names[dt.month0() as usize]);
            }
            FormatToken::MonthLetter => {
                let month_letter_fixed = match dt.month() {
                    // month() is 1-based
                    1 => "J",
                    2 => "F",
                    3 => "M",
                    4 => "A",
                    5 => "M",
                    6 => "J",
                    7 => "J",
                    8 => "A",
                    9 => "S",
                    10 => "O",
                    11 => "N",
                    12 => "D",
                    _ => "", // Should not happen with valid NaiveDateTime
                };
                result.push_str(month_letter_fixed);
            }
            FormatToken::DayNum => {
                result.push_str(&dt.day().to_string());
            }
            FormatToken::DayNumPadded => {
                result.push_str(&format!("{:02}", dt.day()));
            }
            FormatToken::WeekdayAbbr => {
                result.push_str(
                    &locale.short_day_names[dt.weekday().num_days_from_sunday() as usize],
                );
            }
            FormatToken::WeekdayFullName => {
                result.push_str(&locale.day_names[dt.weekday().num_days_from_sunday() as usize]);
            }

            // Time tokens
            FormatToken::Hour12Or24 => {
                let hour = dt.hour();
                if has_ampm {
                    let hour12 = if hour == 0 || hour == 12 {
                        12
                    } else {
                        hour % 12
                    };
                    result.push_str(&hour12.to_string());
                } else {
                    result.push_str(&hour.to_string());
                }
            }
            FormatToken::Hour12Or24Padded => {
                let hour = dt.hour();
                if has_ampm {
                    let hour12 = if hour == 0 || hour == 12 {
                        12
                    } else {
                        hour % 12
                    };
                    result.push_str(&format!("{:02}", hour12));
                } else {
                    result.push_str(&format!("{:02}", hour));
                }
            }
            FormatToken::MinuteNum => {
                result.push_str(&dt.minute().to_string());
            }
            FormatToken::MinuteNumPadded => {
                result.push_str(&format!("{:02}", dt.minute()));
            }

            FormatToken::SecondNum | FormatToken::SecondNumPadded => {
                let mut sec_to_display = dt.second();
                let nanos = dt.nanosecond();

                // Check if fractional seconds (e.g., .0, .00) will be formatted immediately after this token.
                let mut will_format_fractions = false;
                if let Some(FormatToken::LiteralChar('.')) = tokens_iter.peek() {
                    let mut temp_iter = tokens_iter.clone();
                    temp_iter.next(); // Consume the tentative '.'
                    if let Some(FormatToken::DigitOrZero) = temp_iter.peek() {
                        will_format_fractions = true;
                    }
                }

                if !will_format_fractions && nanos >= 500_000_000 {
                    sec_to_display += 1;
                    if sec_to_display == 60 {
                        sec_to_display = 0;
                        // Note: This simple rollover for display doesn't propagate to minutes/hours here.
                        // This matches Excel's typical behavior for 'ss' display rounding.
                    }
                }

                if matches!(token, FormatToken::SecondNumPadded) {
                    result.push_str(&format!("{:02}", sec_to_display));
                } else {
                    result.push_str(&sec_to_display.to_string());
                }

                // Fractional seconds logic (now respects will_format_fractions)
                if will_format_fractions {
                    // This block is entered only if we peeked LiteralChar('.') and then DigitOrZero.
                    // So, we must consume the actual LiteralChar('.') from the main iterator.
                    if let Some(FormatToken::LiteralChar('.')) = tokens_iter.peek() {
                        // Re-check for safety
                        tokens_iter.next(); // Consume the '.'
                        result.push('.');

                        let mut num_placeholders = 0;
                        let mut count_iter = tokens_iter.clone();
                        while let Some(FormatToken::DigitOrZero) = count_iter.peek() {
                            count_iter.next();
                            num_placeholders += 1;
                            if num_placeholders >= 9 {
                                break;
                            }
                        }

                        if num_placeholders > 0 {
                            let mut nanos_val = dt.nanosecond(); // Use original nanos for fraction
                            for i in 0..num_placeholders {
                                let divisor = 10u32.pow(8 - i as u32);
                                let digit = nanos_val / divisor;
                                result.push_str(&digit.to_string());
                                nanos_val %= divisor;
                                if let Some(FormatToken::DigitOrZero) = tokens_iter.peek() {
                                    tokens_iter.next(); // Consume the placeholder from main iterator
                                } else {
                                    break;
                                }
                            }
                        }
                    } // End consuming actual '.'
                } // End fractional seconds block
            }
            FormatToken::AmPm(style) => {
                let marker = if dt.hour() < 12 {
                    &locale.ampm_markers[0]
                } else {
                    &locale.ampm_markers[1]
                };
                match style {
                    AmPmStyle::UpperCase => result.push_str(&marker.to_uppercase()),
                    AmPmStyle::LowerCase => result.push_str(&marker.to_lowercase()),
                }
            }
            FormatToken::AP(style) => {
                let base_marker = if dt.hour() < 12 {
                    &locale.ampm_markers[0]
                } else {
                    &locale.ampm_markers[1]
                };
                if let Some(char_to_push) = base_marker.chars().next() {
                    match style {
                        AmPmStyle::UpperCase => {
                            result.push(char_to_push.to_uppercase().next().unwrap_or(char_to_push))
                        }
                        AmPmStyle::LowerCase => {
                            result.push(char_to_push.to_lowercase().next().unwrap_or(char_to_push))
                        }
                    }
                }
            }

            FormatToken::ElapsedHours => {
                result.push_str("[h]"); /* TODO: Requires duration logic */
            }
            FormatToken::ElapsedMinutes => {
                result.push_str("[m]"); /* TODO: Requires duration logic */
            }
            FormatToken::ElapsedSeconds => {
                result.push_str("[s]"); /* TODO: Requires duration logic */
            }

            FormatToken::MonthOrMinute1 => {
                /* Parser should resolve; assuming month for now */
                result.push_str(&dt.month().to_string());
            }
            FormatToken::MonthOrMinute2 => {
                /* Parser should resolve; assuming padded month */
                result.push_str(&format!("{:02}", dt.month()));
            }

            FormatToken::LiteralChar(c) => result.push(*c),
            FormatToken::QuotedText(text) => result.push_str(text),
            FormatToken::SkipWidth(_) => result.push(' '),
            _ => {}
        }
    }

    // If result is empty, it means only datetime tokens that are still TODO were present.
    // Or all tokens were handled but produced nothing (e.g. format was just "yyyy" and year was 0 for some reason).
    // If the format string was empty or only contained unhandled placeholders, result might be empty.
    if result.is_empty() && !section.tokens.is_empty() {
        // Fallback if all were TODOs, return a representation of the converted datetime for debugging
        return format!("DT_CONVERTED: {:?}", dt);
    }
    // If section.tokens was empty, result is empty, and that's fine (empty format section).

    result
}

// New function to format durations like [h]:mm:ss
pub(super) fn format_duration(
    value: f64, // Excel serial date/time value
    section: &FormatSection,
    _locale: &LocaleSettings, // Placeholder for future use
) -> String {
    let mut result = String::new();

    if value < 0.0 {
        return format!(
            "ERROR: Negative value ({}) not allowed for duration format.",
            value
        );
    }

    let total_seconds_float = value * 86400.0;
    let total_seconds_rounded = total_seconds_float.round() as i64;

    // Elapsed parts are based on rounded total seconds
    let hours_for_h_token = total_seconds_rounded / 3600;
    let minutes_for_m_token = total_seconds_rounded / 60;
    let seconds_for_s_token = total_seconds_rounded;

    // Time parts (mm, ss) are also based on rounded total seconds for display consistency
    let minutes_part_for_mm_token = (total_seconds_rounded / 60) % 60;
    let seconds_part_for_ss_token = total_seconds_rounded % 60;

    // Fractional seconds are based on the original float's fraction
    let nanos_part_for_dot0_token = (total_seconds_float.fract() * 1_000_000_000.0).round() as u32;

    let mut tokens_iter = section.tokens.iter().peekable();
    while let Some(token) = tokens_iter.next() {
        match token {
            FormatToken::ElapsedHours => {
                result.push_str(&hours_for_h_token.to_string());
            }
            FormatToken::ElapsedMinutes => {
                result.push_str(&minutes_for_m_token.to_string());
            }
            FormatToken::ElapsedSeconds => {
                result.push_str(&seconds_for_s_token.to_string());
            }
            // Handle MonthOrMinute1/2 as minutes in duration context if parser didn't resolve
            FormatToken::MonthOrMinute1 => {
                // Treat as MinuteNum in duration context
                result.push_str(&minutes_part_for_mm_token.to_string());
            }
            FormatToken::MinuteNum => {
                result.push_str(&minutes_part_for_mm_token.to_string());
            }
            FormatToken::MonthOrMinute2 => {
                // Treat as MinuteNumPadded in duration context
                result.push_str(&format!("{:02}", minutes_part_for_mm_token));
            }
            FormatToken::MinuteNumPadded => {
                result.push_str(&format!("{:02}", minutes_part_for_mm_token));
            }
            FormatToken::SecondNum | FormatToken::SecondNumPadded => {
                if matches!(token, FormatToken::SecondNumPadded) {
                    result.push_str(&format!("{:02}", seconds_part_for_ss_token));
                } else {
                    result.push_str(&seconds_part_for_ss_token.to_string());
                }

                // Fractional seconds logic (similar to format_datetime)
                if let Some(FormatToken::LiteralChar('.')) = tokens_iter.peek() {
                    tokens_iter.next(); // Consume '.'
                    result.push('.');

                    let mut num_placeholders = 0;
                    let mut count_iter = tokens_iter.clone();
                    while let Some(FormatToken::DigitOrZero) = count_iter.peek() {
                        count_iter.next();
                        num_placeholders += 1;
                        if num_placeholders >= 9 {
                            break;
                        }
                    }

                    if num_placeholders > 0 {
                        let mut current_nanos = nanos_part_for_dot0_token;
                        for i in 0..num_placeholders {
                            let divisor = 10u32.pow(8 - i as u32);
                            let digit = current_nanos / divisor;
                            result.push_str(&digit.to_string());
                            current_nanos %= divisor;
                            if let Some(FormatToken::DigitOrZero) = tokens_iter.peek() {
                                tokens_iter.next();
                            } else {
                                break;
                            }
                        }
                    }
                }
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

// Further helper functions for each token type would go here, e.g.:
// fn format_year(year: i32, format: &YearFormat, locale: &LocaleSettings) -> String { ... }
// fn format_month(month: u32, format: &MonthFormat, locale: &LocaleSettings) -> String { ... }
// etc.
