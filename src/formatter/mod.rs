//! Number format formatting module
//!
//! This module is responsible for formatting numbers according to parsed number format patterns.
//! The main entry point is the `format_number` function.
//! Number formatting implementation
//!
//! This module implements formatting of numbers according to parsed number format patterns.

use crate::types::{ExponentialNotation, FormatSection, FormatToken, LocaleSettings, NumberFormat};
use std::fmt::Write;

/// Format a number according to the specified number format pattern
///
/// # Arguments
/// * `value` - The numeric value to format
/// * `format` - The parsed number format pattern
/// * `locale` - Locale settings for formatting
///
/// # Returns
/// * `String` - The formatted number string
///
/// # Examples
/// ```
/// use number_format::formatter::format_number;
/// use number_format::parser::parse_number_format;
/// use number_format::types::LocaleSettings;
///
/// let format = parse_number_format("0.00").unwrap();
/// let result = format_number(123.456, &format, &LocaleSettings::default());
/// assert_eq!(result, "123.46");
/// ```
pub fn format_number(value: f64, format: &NumberFormat, locale: &LocaleSettings) -> String {
    // Handle special cases first: text value
    if value.is_nan() && format.text_section.is_some() {
        return format_text("NaN", format);
    }

    // Determine which section to use based on value and conditions
    let section = select_section(value, format);

    // Format the number using the selected section
    format_value(value, section, locale)
}

/// Format a text value according to the specified number format pattern
///
/// # Arguments
/// * `text` - The text value to format
/// * `format` - The parsed number format pattern
///
/// # Returns
/// * `String` - The formatted text string
pub fn format_text(text: &str, format: &NumberFormat) -> String {
    if let Some(text_section) = &format.text_section {
        format_text_with_section(text, text_section)
    } else {
        // If no text section is defined, return the text as is
        text.to_string()
    }
}

/// Select the appropriate format section based on the value and format conditions
fn select_section(value: f64, format: &NumberFormat) -> &FormatSection {
    // Check for conditional sections first
    if let Some(condition) = &format.positive_section.condition {
        let matches = match condition.operator {
            crate::types::ComparisonOperator::Eq => value == condition.value,
            crate::types::ComparisonOperator::Gt => value > condition.value,
            crate::types::ComparisonOperator::Lt => value < condition.value,
            crate::types::ComparisonOperator::Ge => value >= condition.value,
            crate::types::ComparisonOperator::Le => value <= condition.value,
            crate::types::ComparisonOperator::Ne => value != condition.value,
        };

        if matches {
            return &format.positive_section;
        }
    }

    if let Some(section) = &format.negative_section {
        if let Some(condition) = &section.condition {
            let matches = match condition.operator {
                crate::types::ComparisonOperator::Eq => value == condition.value,
                crate::types::ComparisonOperator::Gt => value > condition.value,
                crate::types::ComparisonOperator::Lt => value < condition.value,
                crate::types::ComparisonOperator::Ge => value >= condition.value,
                crate::types::ComparisonOperator::Le => value <= condition.value,
                crate::types::ComparisonOperator::Ne => value != condition.value,
            };

            if matches {
                return section;
            }
        }
    }

    if let Some(section) = &format.zero_section {
        if let Some(condition) = &section.condition {
            let matches = match condition.operator {
                crate::types::ComparisonOperator::Eq => value == condition.value,
                crate::types::ComparisonOperator::Gt => value > condition.value,
                crate::types::ComparisonOperator::Lt => value < condition.value,
                crate::types::ComparisonOperator::Ge => value >= condition.value,
                crate::types::ComparisonOperator::Le => value <= condition.value,
                crate::types::ComparisonOperator::Ne => value != condition.value,
            };

            if matches {
                return section;
            }
        }
    }

    // If no conditions matched or no conditional sections defined,
    // use standard sign-based selection
    if value < 0.0 {
        if let Some(section) = &format.negative_section {
            if section.condition.is_none() {
                return section;
            }
        }
    } else if value == 0.0 {
        if let Some(section) = &format.zero_section {
            if section.condition.is_none() {
                return section;
            }
        }
    }

    // Default to positive section
    &format.positive_section
}

/// Format a numeric value using the specified format section
fn format_value(value: f64, section: &FormatSection, locale: &LocaleSettings) -> String {
    let mut result = String::new();

    // NEW: Check for text-only output mode
    let is_text_output_mode = !section.tokens.iter().any(|token| {
        matches!(
            token,
            FormatToken::DigitOrZero
                | FormatToken::DigitIfNeeded
                | FormatToken::DigitOrSpace
                | FormatToken::DecimalPoint
                | FormatToken::Percentage
                | FormatToken::Exponential(_)
                | FormatToken::TextValue // If @ is present, it's not pure text for a number input
                                         // Date/time tokens might also imply non-text output if section is chosen for a number
        )
    });

    if is_text_output_mode {
        for token in &section.tokens {
            match token {
                FormatToken::LiteralChar(c) => {
                    result.push(*c);
                }
                FormatToken::QuotedText(text) => {
                    result.push_str(text);
                }
                _ => {}
            }
        }
        return result;
    }

    // Determine if we need to apply percentage
    let has_percentage = section
        .tokens
        .iter()
        .any(|t| matches!(t, FormatToken::Percentage));
    let abs_value = value.abs();
    let adjusted_value = if has_percentage {
        abs_value * 100.0
    } else {
        abs_value
    };

    // Handle exponential notation if present
    if let Some(exp_token_idx) = section
        .tokens
        .iter()
        .position(|t| matches!(t, FormatToken::Exponential(_)))
    {
        return format_exponential(value, section, exp_token_idx, locale);
    }

    // 基础值处理
    let is_negative = value < 0.0;
    let uses_parentheses = section.tokens.iter().any(|t| {
        matches!(t, FormatToken::LiteralChar('(')) || matches!(t, FormatToken::LiteralChar(')'))
    });

    // 获取整数和小数部分
    let integer_part = adjusted_value.trunc() as i64;
    let decimal_part = adjusted_value.fract();

    // 将整数转为字符数组
    let integer_str = integer_part.to_string();
    let int_digits: Vec<char> = integer_str.chars().collect();

    // 小数部分处理
    let mut decimal_digits = Vec::new();
    let mut decimal_places = 0;

    // 计算需要的小数位数
    let mut after_decimal = false;
    for token in &section.tokens {
        if after_decimal
            && matches!(
                token,
                FormatToken::DigitOrZero | FormatToken::DigitIfNeeded | FormatToken::DigitOrSpace
            )
        {
            decimal_places += 1;
        } else if matches!(token, FormatToken::DecimalPoint) {
            after_decimal = true;
        }
    }

    // 计算小数位数
    let mut remaining_decimal = decimal_part;
    for _ in 0..decimal_places {
        remaining_decimal *= 10.0;
        let digit = remaining_decimal.trunc() as i32;
        decimal_digits.push(digit);
        remaining_decimal -= digit as f64;
    }

    // 处理舍入
    const EPSILON: f64 = 1e-9;
    if remaining_decimal >= (0.5 - EPSILON) && decimal_places > 0 && !decimal_digits.is_empty() {
        let last_idx = decimal_digits.len() - 1;
        decimal_digits[last_idx] += 1;

        // 处理进位
        for i in (0..=last_idx).rev() {
            if decimal_digits[i] >= 10 {
                decimal_digits[i] -= 10;
                if i > 0 {
                    decimal_digits[i - 1] += 1;
                } else {
                    // 进位到整数部分
                    let new_integer_part = integer_part + 1;
                    // 更新整数部分，重新格式化
                    return format_value(
                        if is_negative {
                            -new_integer_part as f64
                        } else {
                            new_integer_part as f64
                        },
                        section,
                        locale,
                    );
                }
            } else {
                break;
            }
        }
    }

    // 构建最终结果

    // Determine if thousands separators should be applied for this section
    let should_apply_thousands_separator = section
        .tokens
        .iter()
        .any(|token| matches!(token, FormatToken::ThousandsSeparator));

    let mut formatted_integer_part_vec: Vec<char>;
    if should_apply_thousands_separator && !int_digits.is_empty() {
        formatted_integer_part_vec =
            Vec::with_capacity(int_digits.len() + (int_digits.len() - 1) / 3);
        let mut count = 0;
        for (i, digit) in int_digits.iter().rev().enumerate() {
            if i > 0 && count % 3 == 0 {
                formatted_integer_part_vec.push(locale.thousands_separator);
            }
            formatted_integer_part_vec.push(*digit);
            count += 1;
        }
        formatted_integer_part_vec.reverse(); // Reverse back to correct order
    } else {
        formatted_integer_part_vec = int_digits.to_vec(); // Use original digits if no separator
    }

    let mut int_digits_iter = formatted_integer_part_vec.iter().cloned().peekable();
    let mut sign_printed = false;
    let mut in_decimal_part = false;
    let mut frac_pos = 0; // For indexing decimal_digits

    // Pre-calculate for integer part formatting
    let mut total_integer_placeholders: usize = 0;
    let mut first_decimal_point_idx: Option<usize> = None;
    for (idx, token) in section.tokens.iter().enumerate() {
        if matches!(token, FormatToken::DecimalPoint) && first_decimal_point_idx.is_none() {
            first_decimal_point_idx = Some(idx);
        }
        if (first_decimal_point_idx.is_none() || idx < first_decimal_point_idx.unwrap())
            && matches!(
                token,
                FormatToken::DigitOrZero | FormatToken::DigitIfNeeded | FormatToken::DigitOrSpace
            )
        {
            total_integer_placeholders += 1;
        }
    }

    // let num_actual_int_digits = int_digits.len(); // OLD
    // Use the length of the (potentially) separator-formatted integer part for placeholder calculations
    // No, num_actual_int_digits should still refer to the raw number of digits for padding calculation.
    // The padding is for the *number* of digits, not the length of the formatted string.
    let num_actual_raw_int_digits = int_digits.len();

    let padding_len = total_integer_placeholders.saturating_sub(num_actual_raw_int_digits);

    let mut current_int_placeholder_idx = 0; // 0-indexed, among integer placeholders
    let mut actual_int_digit_printed = false;

    let mut temp_leading_int_digits_buffer = String::new(); // Buffer for digits longer than placeholders

    for token in &section.tokens {
        match token {
            FormatToken::LiteralChar(c) => {
                if !sign_printed && is_negative {
                    if uses_parentheses {
                        if *c == '(' {
                            result.push('(');
                            sign_printed = true;
                            continue;
                        }
                    } else if *c == '-' {
                        // If '-' is a literal in negative section
                        result.push('-');
                        sign_printed = true;
                        continue;
                    }
                }
                // Print pending leading integer digits if any, before a literal
                if !temp_leading_int_digits_buffer.is_empty() {
                    result.push_str(&temp_leading_int_digits_buffer);
                    temp_leading_int_digits_buffer.clear();
                    actual_int_digit_printed = true;
                }
                while int_digits_iter.peek().is_some()
                    && (current_int_placeholder_idx >= total_integer_placeholders)
                    && !in_decimal_part
                {
                    if !sign_printed && is_negative && !uses_parentheses {
                        result.push('-');
                        sign_printed = true;
                    }
                    result.push(int_digits_iter.next().unwrap());
                    actual_int_digit_printed = true;
                }

                result.push(*c);
                if uses_parentheses && is_negative && *c == ')' && !sign_printed {
                    // This case is tricky: if '(' was not a token but ')' is.
                    // Assume if ')' is a token, it should be printed.
                    // Proper sign printing with parentheses requires '(' and ')' to be tokens.
                }
            }
            FormatToken::QuotedText(text) => {
                // Print pending leading integer digits
                if !temp_leading_int_digits_buffer.is_empty() {
                    result.push_str(&temp_leading_int_digits_buffer);
                    temp_leading_int_digits_buffer.clear();
                    actual_int_digit_printed = true;
                }
                while int_digits_iter.peek().is_some()
                    && (current_int_placeholder_idx >= total_integer_placeholders)
                    && !in_decimal_part
                {
                    if !sign_printed && is_negative && !uses_parentheses {
                        result.push('-');
                        sign_printed = true;
                    }
                    result.push(int_digits_iter.next().unwrap());
                    actual_int_digit_printed = true;
                }
                result.push_str(text);
            }
            FormatToken::DecimalPoint => {
                if !sign_printed && is_negative && !uses_parentheses {
                    result.push('-');
                    sign_printed = true;
                }
                // Print all remaining integer digits from iter that were not consumed by placeholders
                // or that exceeded placeholders.
                if !temp_leading_int_digits_buffer.is_empty() {
                    result.push_str(&temp_leading_int_digits_buffer);
                    temp_leading_int_digits_buffer.clear();
                    actual_int_digit_printed = true;
                }
                while let Some(digit) = int_digits_iter.next() {
                    if !actual_int_digit_printed
                        && !sign_printed
                        && is_negative
                        && !uses_parentheses
                    {
                        result.push('-');
                        sign_printed = true;
                    }
                    result.push(digit);
                    actual_int_digit_printed = true;
                }
                if !actual_int_digit_printed && integer_part == 0 {
                    let has_zero_placeholder_for_int = section
                        .tokens
                        .iter()
                        .take_while(|t| !matches!(t, FormatToken::DecimalPoint))
                        .any(|t| matches!(t, FormatToken::DigitOrZero));
                    if has_zero_placeholder_for_int || total_integer_placeholders == 0 {
                        if !sign_printed && is_negative && !uses_parentheses {
                            result.push('-');
                            sign_printed = true;
                        }
                        result.push('0');
                        actual_int_digit_printed = true;
                    }
                }

                result.push(locale.decimal_point); // USE LOCALE DECIMAL POINT
                in_decimal_part = true;
            }
            FormatToken::DigitOrZero | FormatToken::DigitIfNeeded | FormatToken::DigitOrSpace => {
                if !in_decimal_part {
                    // INTEGER PART
                    if !sign_printed && is_negative && !uses_parentheses {
                        result.push('-');
                        sign_printed = true;
                    }
                    if !temp_leading_int_digits_buffer.is_empty() {
                        // Digits longer than placeholders first
                        result.push_str(&temp_leading_int_digits_buffer);
                        temp_leading_int_digits_buffer.clear();
                        actual_int_digit_printed = true;
                    }

                    let mut char_to_print: Option<char> = None;
                    let mut consumed_digit_this_turn = false;

                    if current_int_placeholder_idx < padding_len {
                        // Padding part
                        match token {
                            FormatToken::DigitOrZero => char_to_print = Some('0'),
                            FormatToken::DigitOrSpace => char_to_print = Some(' '),
                            FormatToken::DigitIfNeeded => {} // No char for # in padding
                            _ => unreachable!(),
                        }
                    } else {
                        // Actual digit mapping part
                        if let Some(digit_char) = int_digits_iter.peek().cloned() {
                            match token {
                                FormatToken::DigitOrZero => {
                                    char_to_print = Some(digit_char);
                                    consumed_digit_this_turn = true;
                                }
                                FormatToken::DigitIfNeeded => {
                                    if actual_int_digit_printed
                                        || digit_char != '0'
                                        || (num_actual_raw_int_digits == 1 && integer_part == 0)
                                    {
                                        char_to_print = Some(digit_char);
                                    } else {
                                        // Suppress leading zero for #, char_to_print remains None
                                    }
                                    consumed_digit_this_turn = true; // Consume even if not printed for #
                                }
                                FormatToken::DigitOrSpace => {
                                    char_to_print = Some(digit_char);
                                    consumed_digit_this_turn = true;
                                }
                                _ => unreachable!(),
                            }
                        } else {
                            // No more actual digits, but placeholder exists
                            match token {
                                FormatToken::DigitOrZero => char_to_print = Some('0'),
                                FormatToken::DigitIfNeeded => {}
                                FormatToken::DigitOrSpace => char_to_print = Some(' '),
                                _ => unreachable!(),
                            }
                        }
                    }

                    if let Some(p_char) = char_to_print {
                        result.push(p_char);
                        if consumed_digit_this_turn && p_char.is_ascii_digit()
                            || (matches!(token, FormatToken::DigitOrZero)
                                && p_char == '0'
                                && current_int_placeholder_idx < padding_len)
                        {
                            // If we printed a digit from input, or a padding '0' that behaves like a digit
                            actual_int_digit_printed = true;
                        }
                    }
                    if consumed_digit_this_turn {
                        int_digits_iter.next();
                    }
                    current_int_placeholder_idx += 1;
                } else {
                    // DECIMAL PART
                    if frac_pos < decimal_digits.len() {
                        match token {
                            FormatToken::DigitOrZero => {
                                write!(result, "{}", decimal_digits[frac_pos]).unwrap();
                            }
                            FormatToken::DigitIfNeeded => {
                                let digit = decimal_digits[frac_pos];
                                if digit != 0 || frac_pos < decimal_digits.len() - 1 {
                                    write!(result, "{}", digit).unwrap();
                                }
                            }
                            FormatToken::DigitOrSpace => {
                                write!(result, "{}", decimal_digits[frac_pos]).unwrap();
                            }
                            _ => unreachable!(),
                        }
                    } else {
                        // Decimal digits from number exhausted, pad with format
                        match token {
                            FormatToken::DigitOrZero => result.push('0'),
                            FormatToken::DigitIfNeeded => {}
                            FormatToken::DigitOrSpace => result.push(' '),
                            _ => unreachable!(),
                        }
                    }
                    frac_pos += 1;
                }
            }
            FormatToken::Percentage => {
                if !sign_printed && is_negative && !uses_parentheses {
                    result.push('-');
                    sign_printed = true;
                }
                if !temp_leading_int_digits_buffer.is_empty() {
                    result.push_str(&temp_leading_int_digits_buffer);
                    temp_leading_int_digits_buffer.clear();
                    actual_int_digit_printed = true;
                }
                for digit in int_digits_iter.by_ref() {
                    if !actual_int_digit_printed
                        && !sign_printed
                        && is_negative
                        && !uses_parentheses
                    {
                        result.push('-');
                        sign_printed = true;
                    }
                    result.push(digit);
                    actual_int_digit_printed = true;
                }
                if !actual_int_digit_printed && integer_part == 0 {
                    // e.g. format "0%" for value 0.0 should be "0%"
                    let has_zero_placeholder_for_int = section
                        .tokens
                        .iter()
                        .take_while(|t| !matches!(t, FormatToken::Percentage)) // Check tokens before %
                        .any(|t| matches!(t, FormatToken::DigitOrZero) && !in_decimal_part); // Crude check for int '0'
                    if has_zero_placeholder_for_int || total_integer_placeholders == 0 {
                        if !sign_printed && is_negative && !uses_parentheses {
                            result.push('-');
                            sign_printed = true;
                        }
                        result.push('0');
                        actual_int_digit_printed = true;
                    }
                }
                result.push('%');
            }
            FormatToken::Fill(_) | FormatToken::SkipWidth(_) => {
                // These are typically for alignment and might need special handling
                // For now, let's assume they are like literals or do nothing concrete for value output
            }
            FormatToken::Color(_) => {
                // Colors do not produce output in the string
            }
            FormatToken::ThousandsSeparator => {
                // If by some logic, an explicit comma in the format string needs to force a sign print before it.
                if !sign_printed && is_negative && !uses_parentheses {
                    result.push('-');
                    sign_printed = true;
                }
                // The actual separator is part of int_digits_iter if enabled.
                // This token in the format string just signals to enable the feature.
                // If it *is* printed (e.g. if int_digits_iter somehow didn't have it), ensure it's the locale one.
                // This branch might be redundant if int_digits_iter is correctly populated.
            }
            // Other date/time tokens are not expected in format_value, but in a full formatter
            _ => {
                // Potentially Year, Month, Day etc. if sections were mixed.
                // For now, assume these are not hit in pure numeric formatting.
            }
        }
    }

    // Final flush of any remaining integer digits if not consumed by placeholders and not followed by decimal/percentage
    if !temp_leading_int_digits_buffer.is_empty() {
        result.push_str(&temp_leading_int_digits_buffer);
        temp_leading_int_digits_buffer.clear();
        actual_int_digit_printed = true;
    }
    for digit in int_digits_iter {
        if !actual_int_digit_printed && !sign_printed && is_negative && !uses_parentheses {
            result.push('-');
            sign_printed = true;
        }
        result.push(digit);
        actual_int_digit_printed = true; // Ensure this is set
    }

    // If result is still empty, and it was a number like 0 formatted with only "#", result might be empty.
    // Excel shows "0" if format is "#" and value is 0.
    // Or if format is "" (empty) and value is 0 -> "0"
    // If no digits were printed at all, and value is 0, print "0"
    if !actual_int_digit_printed && value == 0.0 && !result.contains("-") && !result.contains("(") {
        // Avoid -0 if sign already handled
        // Check if format was not just "text"
        let is_text_only_format = section
            .tokens
            .iter()
            .all(|t| matches!(t, FormatToken::QuotedText(_) | FormatToken::LiteralChar(_)));
        if !is_text_only_format {
            result.push('0');
        }
    }

    // Handle sign for () if not already done by literal '('
    if is_negative && uses_parentheses && !sign_printed {
        result.insert(0, '(');
        result.push(')');
    }

    result
}

/// Format a text value with a text section
fn format_text_with_section(text: &str, section: &FormatSection) -> String {
    let mut result = String::new();

    for token in &section.tokens {
        match token {
            FormatToken::TextValue => {
                result.push_str(text);
            }
            FormatToken::LiteralChar(c) => {
                result.push(*c);
            }
            FormatToken::QuotedText(quoted_text) => {
                result.push_str(quoted_text);
            }
            _ => {
                // Ignore other tokens in text section
            }
        }
    }

    result
}

/// Format a number in exponential notation
fn format_exponential(
    value: f64,
    section: &FormatSection,
    exp_token_idx: usize,
    locale: &LocaleSettings,
) -> String {
    let mut result = String::new();

    // Get the exponential token
    let exp_token = &section.tokens[exp_token_idx];
    let _exp_notation_type = match exp_token {
        // Renamed to avoid unused var warning if only sign matters
        FormatToken::Exponential(notation) => notation,
        _ => unreachable!(), // Should be caught by caller
    };

    // Format with scientific notation
    let abs_value = value.abs();
    let (mantissa, exponent) = if abs_value == 0.0 {
        (0.0, 0)
    } else {
        let log10_val = abs_value.log10();
        let exponent_val = log10_val.floor();
        let mantissa_val = abs_value / 10.0_f64.powf(exponent_val);
        (mantissa_val, exponent_val as i32)
    };

    // Format mantissa part with proper precision
    let is_negative = value < 0.0;
    let sign = if is_negative { "-" } else { "" };

    // Count number of desired decimal places in mantissa
    let mut mantissa_precision = 0; // Default to 0, meaning only the integer part of mantissa if no frac part in format
    let mut in_mantissa_decimal_part = false;
    for token in section.tokens.iter().take(exp_token_idx) {
        if matches!(token, FormatToken::DecimalPoint) {
            in_mantissa_decimal_part = true;
            continue;
        }
        if in_mantissa_decimal_part
            && matches!(
                token,
                FormatToken::DigitOrZero | FormatToken::DigitIfNeeded | FormatToken::DigitOrSpace
            )
        {
            mantissa_precision += 1;
        }
    }
    // If no decimal point was found before E, check if there are integer placeholders before E
    // Excel default: For "0E+00", 12345 -> 1E+04 (no decimals in mantissa)
    // For "0.00E+00", 12345 -> 1.23E+04 (2 decimals in mantissa)
    // So, mantissa_precision calculation above based on tokens after decimal point and before E is correct.
    // If mantissa_precision is still 0, it means format like "0E+00".

    // Round mantissa correctly based on its desired precision
    let power = 10.0_f64.powi(mantissa_precision as i32);
    let rounded_mantissa = (mantissa * power).round() / power;

    // Adjust exponent if rounding mantissa caused it to become >= 10 or < 1
    let (final_mantissa, final_exponent) = if rounded_mantissa == 0.0 {
        // handle 0.0 case separately
        (0.0, 0)
    } else if rounded_mantissa >= 10.0 {
        (rounded_mantissa / 10.0, exponent + 1)
    } else if rounded_mantissa < 1.0 && mantissa != 0.0 {
        // mantissa != 0.0 to avoid 0.0 becoming 0.0 E-1
        // This case needs care: if format is 0E+00, and value is 0.123 -> 1E-01
        // if format is 0.0E+00, and value is 0.0123 -> 1.2E-02
        // The initial mantissa calculation (abs_value / 10.0_f64.powf(exponent_val)) ensures mantissa >=1 and <10
        // So, rounding alone should not make it < 1 unless original value was very small and precision is low.
        // If it does become < 1 due to rounding (e.g. 1.0000xxx rounded to 0 precision -> 1.0, but if 0.5 rounded to 0 precision -> 1.0. If 0.4 -> 0.0)
        // Let's stick to initial mantissa/exponent adjustment and rely on precision for rounding.
        (rounded_mantissa, exponent) // Revisit if exponent adjustment for mantissa < 1 due to rounding is needed
    } else {
        (rounded_mantissa, exponent)
    };

    write!(result, "{}", sign).unwrap();

    let mut mantissa_str = format!(
        "{:.precision$}",
        final_mantissa,
        precision = mantissa_precision
    );
    if locale.decimal_point != '.' {
        mantissa_str = mantissa_str.replace('.', &locale.decimal_point.to_string());
    }
    write!(result, "{}", mantissa_str).unwrap();

    // Add E notation
    let final_exp_sign_str = if final_exponent < 0 {
        "-"
    } else {
        match &section.tokens[exp_token_idx] {
            FormatToken::Exponential(ExponentialNotation::Plus) => "+",
            FormatToken::Exponential(ExponentialNotation::Minus) => "",
            _ => unreachable!(), // Should be caught by caller or parser
        }
    };
    write!(result, "E{}{:02}", final_exp_sign_str, final_exponent.abs()).unwrap();

    result
}
