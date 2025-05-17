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

    // Revert to calculating h,m,s from the unrounded total_seconds_precise
    let hours = (total_seconds_precise / 3600.0).trunc() as u32;
    let minutes = ((total_seconds_precise % 3600.0) / 60.0).trunc() as u32;
    let seconds = (total_seconds_precise % 60.0).trunc() as u32;

    // Nanoseconds part: should be based on the original total_seconds_precise's fractional part
    // to retain original precision for fractional second formatting.
    let nanoseconds = ((total_seconds_precise.fract().abs()) * 1_000_000_000.0).round() as u32;

    // Clamp nanoseconds to max value for NaiveTime::from_hms_nano_opt
    let clamped_nanos = nanoseconds.min(999_999_999);

    let time_part = NaiveTime::from_hms_nano_opt(hours, minutes, seconds, clamped_nanos)?;

    Some(NaiveDateTime::new(current_date_part, time_part))
}

/// Helper function to check if a section contains any date/time point-in-time tokens
pub(super) fn section_is_datetime_point_in_time(section: &FormatSection) -> bool {
    section.tokens.iter().any(|token| {
        matches!(
            token,
            FormatToken::YearTwoDigit
                | FormatToken::YearFourDigit
                | FormatToken::MonthNum
                | FormatToken::MonthNumPadded
                | FormatToken::MonthAbbr
                | FormatToken::MonthFullName
                | FormatToken::MonthLetter
                | FormatToken::DayNum
                | FormatToken::DayNumPadded
                | FormatToken::WeekdayAbbr
                | FormatToken::WeekdayFullName
                | FormatToken::Hour12Or24
                | FormatToken::Hour12Or24Padded
                | FormatToken::MinuteNum
                | FormatToken::MinuteNumPadded
                | FormatToken::SecondNum
                | FormatToken::SecondNumPadded
                | FormatToken::AmPm(_)
                | FormatToken::AP(_)
                | FormatToken::MonthOrMinute1
                | FormatToken::MonthOrMinute2
        )
    })
}

/// Helper function to check if a section contains duration-specific tokens
pub(super) fn section_is_duration(section: &FormatSection) -> bool {
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

    let Some(dt_original) = convert_f64_to_datetime(value) else {
        return format!("INVALID_DATE_SERIAL: {}", value);
    };

    // Determine if the format string requests fractional seconds
    let mut format_has_fractional_seconds = false;
    let mut i = 0;
    while i < section.tokens.len() {
        if matches!(section.tokens[i], FormatToken::DecimalPoint) && i + 1 < section.tokens.len() {
            // Check if the token after DecimalPoint is a DigitOrZero, indicating fractional seconds.
            // Need to ensure we are not mistaking a sequence like ". literal" for fractional seconds.
            // The current DigitOrZero token is used for fractional seconds *and* general number placeholders.
            // For date/time, a DigitOrZero after a DecimalPoint is always fractional seconds.
            if matches!(section.tokens[i + 1], FormatToken::DigitOrZero) {
                format_has_fractional_seconds = true;
                break;
            }
        }
        i += 1;
    }

    // If not formatting fractional seconds, round dt_original to the nearest second for display
    let dt_display = if !format_has_fractional_seconds {
        dt_original
            .checked_add_signed(chrono::Duration::nanoseconds(500_000_000))
            .unwrap_or(dt_original) // Fallback to original if addition fails (highly unlikely)
    } else {
        dt_original
    };

    let mut result = String::new();

    let has_ampm_in_section = section
        .tokens
        .iter()
        .any(|t| matches!(t, FormatToken::AmPm(_) | FormatToken::AP(_)));

    // Find the index of the last Hour12Or24 or Hour12Or24Padded token in the section
    let last_hour_token_index: Option<usize> = section
        .tokens
        .iter()
        .rposition(|t| matches!(t, FormatToken::Hour12Or24 | FormatToken::Hour12Or24Padded));

    // Main token processing loop with manual index management
    let mut current_token_index = 0;
    while current_token_index < section.tokens.len() {
        let token = &section.tokens[current_token_index];

        // Use dt_display for H/M/S and date parts, dt_original for nanos
        match token {
            // Date tokens
            FormatToken::YearTwoDigit => {
                result.push_str(&format!("{:02}", dt_display.year() % 100));
            }
            FormatToken::YearFourDigit => {
                result.push_str(&dt_display.year().to_string());
            }
            FormatToken::MonthNum => {
                result.push_str(&dt_display.month().to_string());
            }
            FormatToken::MonthNumPadded => {
                result.push_str(&format!("{:02}", dt_display.month()));
            }
            FormatToken::MonthAbbr => {
                result.push_str(&locale.short_month_names[dt_display.month0() as usize]);
            }
            FormatToken::MonthFullName => {
                result.push_str(&locale.month_names[dt_display.month0() as usize]);
            }
            FormatToken::MonthLetter => {
                let month_letter_fixed = match dt_display.month() {
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
                result.push_str(&dt_display.day().to_string());
            }
            FormatToken::DayNumPadded => {
                result.push_str(&format!("{:02}", dt_display.day()));
            }
            FormatToken::WeekdayAbbr => {
                result.push_str(
                    &locale.short_day_names[dt_display.weekday().num_days_from_sunday() as usize],
                );
            }
            FormatToken::WeekdayFullName => {
                result.push_str(
                    &locale.day_names[dt_display.weekday().num_days_from_sunday() as usize],
                );
            }

            // Time tokens
            FormatToken::Hour12Or24 => {
                let hour = dt_display.hour();
                if has_ampm_in_section && last_hour_token_index == Some(current_token_index) {
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
                let hour = dt_display.hour();
                if has_ampm_in_section && last_hour_token_index == Some(current_token_index) {
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
                result.push_str(&dt_display.minute().to_string());
            }
            FormatToken::MinuteNumPadded => {
                result.push_str(&format!("{:02}", dt_display.minute()));
            }

            FormatToken::SecondNum | FormatToken::SecondNumPadded => {
                let sec_to_display = dt_display.second();

                if matches!(token, FormatToken::SecondNumPadded) {
                    result.push_str(&format!("{:02}", sec_to_display));
                } else {
                    result.push_str(&sec_to_display.to_string());
                }
            }
            FormatToken::AmPm(style) => {
                let marker = if dt_display.hour() < 12 {
                    // Use dt_display for AM/PM decision
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
                let base_marker = if dt_display.hour() < 12 {
                    // Use dt_display for A/P decision
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
            FormatToken::ElapsedHoursPadded => {
                result.push_str("[hh]"); /* TODO: Requires duration logic */
            }
            FormatToken::ElapsedMinutesPadded => {
                result.push_str("[mm]"); /* TODO: Requires duration logic */
            }
            FormatToken::ElapsedSecondsPadded => {
                result.push_str("[ss]"); /* TODO: Requires duration logic */
            }

            FormatToken::MonthOrMinute1 => {
                /* Parser should resolve; assuming month for now based on dt_display */
                // This logic needs to be aware of the m/mm ambiguity resolution.
                // For now, just using month from dt_display.
                // The ambiguity resolution should ideally set a flag on the token or have specific tokens.
                // Assuming if it reaches here and is MonthOrMinute1, it's contextually a month.
                result.push_str(&dt_display.month().to_string());
            }
            FormatToken::MonthOrMinute2 => {
                /* Parser should resolve; assuming padded month */
                result.push_str(&format!("{:02}", dt_display.month()));
            }

            FormatToken::DecimalPoint => {
                result.push('.');
                let mut frac_digits_to_append = String::new();
                let mut placeholders_processed_count = 0;
                let mut lookahead_idx = current_token_index + 1;

                while lookahead_idx < section.tokens.len()
                    && matches!(section.tokens[lookahead_idx], FormatToken::DigitOrZero)
                {
                    placeholders_processed_count += 1;
                    if placeholders_processed_count > 9 {
                        break;
                    } // Max 9 fractional digits
                    lookahead_idx += 1;
                }

                if placeholders_processed_count > 0 {
                    let mut nanos_val = dt_original.nanosecond(); // Use original dt for nanosecond precision
                    for i in 0..placeholders_processed_count {
                        // Divisor for 9 placeholders: 10^8, 10^7,... 10^0
                        // If we have `p` placeholders, we need digits from nano / 10^(9-1) down to nano / 10^(9-p)
                        // Or, for p=1 (tenths): digit = nano / 10^8
                        // for p=2 (hundredths): second digit = (nano % 10^8) / 10^7
                        // Correct divisor logic: for the k-th placeholder (0-indexed from p-1 placeholders)
                        // E.g. for .000 (3 placeholders)
                        // i=0 (1st placeholder, tenths): nano / 10^8
                        // i=1 (2nd placeholder, hundredths): (nano % 10^8) / 10^7
                        // i=2 (3rd placeholder, thousandths): (nano % 10^7) / 10^6
                        let exponent = 8 - i; // exponent for 10. (8 for 1st digit, 7 for 2nd, etc.)
                        let divisor = 10u32.pow(exponent as u32);
                        let digit = nanos_val / divisor;
                        frac_digits_to_append.push_str(&digit.to_string());
                        nanos_val %= divisor;
                    }
                    result.push_str(&frac_digits_to_append);
                    current_token_index += placeholders_processed_count; // Advance main index past consumed placeholders
                }
            }
            FormatToken::ThousandsSeparator => result.push(','),

            FormatToken::LiteralChar(c) => result.push(*c),
            FormatToken::QuotedText(text) => result.push_str(text),
            FormatToken::SkipWidth(_) => result.push(' '),
            _ => {}
        }
        current_token_index += 1; // Advance to the next token
    }

    // If result is empty, it means only datetime tokens that are still TODO were present.
    // Or all tokens were handled but produced nothing (e.g. format was just "yyyy" and year was 0 for some reason).
    // If the format string was empty or only contained unhandled placeholders, result might be empty.
    if result.is_empty() && !section.tokens.is_empty() {
        // Fallback if all were TODOs, return a representation of the converted datetime for debugging
        return format!("DT_CONVERTED: {:?}", dt_display); // Use dt_display for fallback
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

    // Determine the number of fractional second digits from the format string
    let mut num_frac_sec_digits = 0;
    let mut max_frac_sec_digits_found = 0;
    let mut in_frac_sec_block = false;
    let mut preceded_by_second_token = false;

    for token in &section.tokens {
        match token {
            FormatToken::SecondNum | FormatToken::SecondNumPadded => {
                preceded_by_second_token = true;
                in_frac_sec_block = false; // Reset for new potential s.0 block
                num_frac_sec_digits = 0;
            }
            FormatToken::DecimalPoint => {
                if preceded_by_second_token {
                    in_frac_sec_block = true;
                    num_frac_sec_digits = 0; // Reset count for this new block
                } else {
                    // Decimal point not immediately after s/ss, reset flags
                    in_frac_sec_block = false;
                    preceded_by_second_token = false;
                }
            }
            FormatToken::DigitOrZero => {
                if in_frac_sec_block && preceded_by_second_token {
                    num_frac_sec_digits += 1;
                } else {
                    // DigitOrZero not in a valid s.0 sequence
                    in_frac_sec_block = false;
                    preceded_by_second_token = false; // current token is not 's'
                }
            }
            _ => {
                // Any other token breaks the s.0 sequence
                if in_frac_sec_block {
                    // Update max if we were in a block
                    max_frac_sec_digits_found = max_frac_sec_digits_found.max(num_frac_sec_digits);
                }
                in_frac_sec_block = false;
                preceded_by_second_token = false; // current token is not 's' unless it's an s token itself
                if !matches!(token, FormatToken::SecondNum | FormatToken::SecondNumPadded) {
                    preceded_by_second_token = false;
                }
            }
        }
        if in_frac_sec_block {
            // Continuously update max if still in a valid block
            max_frac_sec_digits_found = max_frac_sec_digits_found.max(num_frac_sec_digits);
        }
    }
    // Final check if format string ends with a frac sec block
    if in_frac_sec_block {
        max_frac_sec_digits_found = max_frac_sec_digits_found.max(num_frac_sec_digits);
    }

    num_frac_sec_digits = max_frac_sec_digits_found.min(9); // Cap at nano precision (Excel typically up to 3)

    let rounded_total_seconds = if num_frac_sec_digits > 0 {
        let rounding_multiplier = 10f64.powi(num_frac_sec_digits);
        (total_seconds_float * rounding_multiplier).round() / rounding_multiplier
    } else {
        total_seconds_float.round() // Round to nearest second if no fractional part in format
    };

    let final_total_seconds_int_part = rounded_total_seconds.trunc() as i64;
    // Ensure nanos are calculated from the (potentially) rounded value's fractional part.
    let final_nanos_part = (rounded_total_seconds.fract().abs() * 1_000_000_000.0).round() as u32;

    let hours_for_h_token = final_total_seconds_int_part / 3600;
    let minutes_for_m_token = final_total_seconds_int_part / 60; // For [m]
    let minutes_part_for_mm_token = (final_total_seconds_int_part / 60) % 60;
    let seconds_part_for_ss_token = final_total_seconds_int_part % 60;

    let mut tokens_iter = section.tokens.iter().peekable();
    while let Some(token) = tokens_iter.next() {
        match token {
            FormatToken::ElapsedHours => {
                result.push_str(&hours_for_h_token.to_string());
            }
            FormatToken::ElapsedHoursPadded => {
                result.push_str(&format!("{:02}", hours_for_h_token));
            }
            FormatToken::ElapsedMinutes => {
                result.push_str(&minutes_for_m_token.to_string());
            }
            FormatToken::ElapsedMinutesPadded => {
                result.push_str(&format!("{:02}", minutes_for_m_token));
            }
            FormatToken::ElapsedSeconds => {
                result.push_str(&final_total_seconds_int_part.to_string());
            }
            FormatToken::ElapsedSecondsPadded => {
                result.push_str(&format!("{:02}", final_total_seconds_int_part));
            }
            FormatToken::MonthOrMinute1 | FormatToken::MinuteNum => {
                result.push_str(&minutes_part_for_mm_token.to_string());
            }
            FormatToken::MonthOrMinute2 | FormatToken::MinuteNumPadded => {
                result.push_str(&format!("{:02}", minutes_part_for_mm_token));
            }
            FormatToken::SecondNum | FormatToken::SecondNumPadded => {
                if matches!(token, FormatToken::SecondNumPadded) {
                    result.push_str(&format!("{:02}", seconds_part_for_ss_token));
                } else {
                    result.push_str(&seconds_part_for_ss_token.to_string());
                }

                if let Some(FormatToken::DecimalPoint) = tokens_iter.peek().copied() {
                    // Changed to check for DecimalPoint
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
                        let mut display_nanos = final_nanos_part;
                        for i in 0..current_frac_sec_placeholders {
                            let divisor = 10u32.pow(8 - i as u32);
                            let digit = display_nanos / divisor;
                            result.push_str(&digit.to_string());
                            display_nanos %= divisor;
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
