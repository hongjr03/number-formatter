use crate::types::{FormatSection, FormatToken, LocaleSettings};

/// Handle special date formats like the non-existent 1900-02-29 (Excel's leap year bug)
pub fn special_dates(
    value: f64,
    section: &FormatSection,
    locale: &LocaleSettings,
) -> Option<String> {
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
        return Some(special_result);
    }
    None
}

/// Format fractional seconds according to the specified precision
pub fn format_fractional_seconds(nanos: u32, precision: usize) -> String {
    let mut result = String::new();

    if precision > 0 {
        let mut nanos_val = nanos;
        for i in 0..precision {
            if i >= 9 {
                break; // Max 9 fractional digits
            }
            // Determine divisor for each position
            let exponent = 8 - i; // exponent for 10. (8 for 1st digit, 7 for 2nd, etc.)
            let divisor = 10u32.pow(exponent as u32);
            let digit = nanos_val / divisor;
            result.push_str(&digit.to_string());
            nanos_val %= divisor;
        }
    }

    result
}

/// Extract currency prefix from tokens if present
pub fn extract_currency_prefix(tokens: &[FormatToken]) -> Option<String> {
    for token in tokens {
        match token {
            // 处理格式 [$US-409]
            FormatToken::CurrencySymbolLocalePrefixed(value) => {
                if let Some(index) = value.find('-') {
                    if index > 0 {
                        return Some(value[..index].to_string());
                    }
                }
            }
            _ => {}
        }
    }
    None
}

/// Extract locale code from tokens if present
#[allow(dead_code)]
pub fn extract_locale_code(tokens: &[FormatToken]) -> Option<String> {
    for token in tokens {
        match token {
            // 处理格式 [$-409] 或 [$US-409] 或 [$€-409]
            FormatToken::CurrencySymbolLocalePrefixed(value) => {
                // 尝试解析可能的格式
                // 1. [$-409] 格式
                if let Some(stripped) = value.strip_prefix("-") {
                    return Some(stripped.to_string());
                }

                // 2. [$US-409] 格式
                if let Some(index) = value.find('-') {
                    if index < value.len() - 1 {
                        return Some(value[index + 1..].to_string());
                    }
                }
            }
            _ => {}
        }
    }
    None
}

/// Count the number of fractional second digits required by the format
pub fn count_fractional_second_digits(section: &FormatSection) -> usize {
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

    max_frac_sec_digits_found.min(9) // Cap at nano precision (Excel typically up to 3)
}

/// Check if format has fractional seconds
pub fn has_fractional_seconds(section: &FormatSection) -> bool {
    let mut i = 0;
    while i < section.tokens.len() {
        if matches!(section.tokens[i], FormatToken::DecimalPoint) && i + 1 < section.tokens.len() {
            // Check if the token after DecimalPoint is a DigitOrZero, indicating fractional seconds.
            // Need to ensure we are not mistaking a sequence like ". literal" for fractional seconds.
            // The current DigitOrZero token is used for fractional seconds *and* general number placeholders.
            // For date/time, a DigitOrZero after a DecimalPoint is always fractional seconds.
            if matches!(section.tokens[i + 1], FormatToken::DigitOrZero) {
                return true;
            }
        }
        i += 1;
    }
    false
}
