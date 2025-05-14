use crate::formatter::datetime;
use crate::formatter::exponential;
use crate::types::{FormatSection, FormatToken, LocaleSettings};
use std::fmt::Write;

/// Helper function to check if a section contains any date/time point-in-time tokens
fn section_is_datetime_point_in_time(section: &FormatSection) -> bool {
    section.tokens.iter().any(|token| {
        matches!(
            token,
            FormatToken::YearTwoDigit |
        FormatToken::YearFourDigit |
        FormatToken::MonthNum |
        FormatToken::MonthNumPadded |
        FormatToken::MonthAbbr |
        FormatToken::MonthFullName |
        FormatToken::MonthLetter |
        FormatToken::DayNum |
        FormatToken::DayNumPadded |
        FormatToken::WeekdayAbbr |
        FormatToken::WeekdayFullName |
        FormatToken::Hour12Or24 |
        FormatToken::Hour12Or24Padded |
        // MinuteNum, SecondNum etc. can appear in durations too, so context is key.
        // AmPm, AP are strong indicators of point-in-time.
        FormatToken::AmPm(_) |
        FormatToken::AP(_) |
        FormatToken::MonthOrMinute1 |
        FormatToken::MonthOrMinute2
        )
    })
}

/// Helper function to check if a section contains duration-specific tokens
fn section_is_duration(section: &FormatSection) -> bool {
    section.tokens.iter().any(|token| {
        matches!(
            token,
            FormatToken::ElapsedHours | FormatToken::ElapsedMinutes | FormatToken::ElapsedSeconds
        )
    })
}

/// Format a numeric value using the specified format section
pub(super) fn format_value(
    original_value_for_sign: f64,
    value_to_format_placeholders: f64,
    section: &FormatSection,
    locale: &LocaleSettings,
    is_positive_section_fallback_for_negative: bool, // True if positive_section is used for a negative original_value
) -> String {
    // Priority: Check for duration formats first, as they use specific tokens like [h], [m], [s]
    if section_is_duration(section) {
        return datetime::format_duration(original_value_for_sign, section, locale);
    }

    // Then check for general date/time (point-in-time) formats
    // Renamed section_is_datetime to section_is_datetime_point_in_time for clarity
    if section_is_datetime_point_in_time(section) {
        return datetime::format_datetime(original_value_for_sign, section, locale);
    }

    // NEW: If section contains @, it overrides ALL other tokens in the section.
    // The output is simply the value as text, with potential sign prepending for fallback negatives.
    if section
        .tokens
        .iter()
        .any(|t| matches!(t, FormatToken::TextValue))
    {
        let mut value_as_text = if is_positive_section_fallback_for_negative {
            // For fallback, use the (abs) value prepared for placeholders.
            value_to_format_placeholders.to_string()
        } else {
            // For non-fallback (e.g. positive section with @, or negative section with @),
            // use the original value, which includes its sign.
            original_value_for_sign.to_string()
        };

        // If this was a positive section used as a fallback for a negative number,
        // and the original value was indeed negative, prepend a minus sign.
        if is_positive_section_fallback_for_negative && original_value_for_sign < 0.0 {
            // value_to_format_placeholders.to_string() should give an unsigned string here.
            // Prepend '-' to achieve the "-" + Format(Abs(Value), PositiveSectionContaining@) behavior.
            if !value_as_text.starts_with('-') {
                // Should be true if value_to_format_placeholders was abs
                value_as_text.insert(0, '-');
            }
        }
        return value_as_text;
    }

    const EPSILON: f64 = 1e-9;
    let mut result = String::new();

    let is_text_output_mode = !section.tokens.iter().any(|token| {
        matches!(
            token,
            FormatToken::DigitOrZero
                | FormatToken::DigitIfNeeded
                | FormatToken::DigitOrSpace
                | FormatToken::DecimalPoint
                | FormatToken::Percentage
                | FormatToken::Exponential(_)
                | FormatToken::TextValue
        )
    });

    if is_text_output_mode {
        for token in &section.tokens {
            match token {
                FormatToken::LiteralChar(c) => result.push(*c),
                FormatToken::QuotedText(text) => result.push_str(text),
                _ => {}
            }
        }
        return result;
    }

    let has_percentage = section
        .tokens
        .iter()
        .any(|t| matches!(t, FormatToken::Percentage));

    // Operate on `value_to_format_placeholders` for all numeric transformations.
    // `value_to_format_placeholders` is expected to be abs(original_value) if is_positive_section_fallback_for_negative is true.
    let abs_value_for_formatting = value_to_format_placeholders.abs(); // Ensure it is absolute for logic below

    let mut adjusted_value = if has_percentage {
        abs_value_for_formatting * 100.0
    } else {
        abs_value_for_formatting
    };

    if section.num_scaling_commas > 0 {
        for _ in 0..section.num_scaling_commas {
            adjusted_value /= 1000.0;
        }
    }

    let mut local_decimal_places = 0;
    let mut after_decimal_flag = false;
    for token in &section.tokens {
        if after_decimal_flag
            && matches!(
                token,
                FormatToken::DigitOrZero | FormatToken::DigitIfNeeded | FormatToken::DigitOrSpace
            )
        {
            local_decimal_places += 1;
        } else if matches!(token, FormatToken::DecimalPoint) {
            after_decimal_flag = true;
        }
    }

    if let Some(exp_token_idx) = section
        .tokens
        .iter()
        .position(|t| matches!(t, FormatToken::Exponential(_)))
    {
        // Exponential formatting should use the value intended for placeholders,
        // but re-apply original sign if not a fallback scenario or if exp needs signed value.
        // For simplicity, assume exponential formatter handles signs internally or `value_to_format_placeholders` is correctly signed for it.
        let value_for_exp =
            if original_value_for_sign < 0.0 && !is_positive_section_fallback_for_negative {
                -adjusted_value // Use negative if it's a negative section call
            } else {
                adjusted_value // Use positive (abs) for positive section or fallback
            };
        return exponential::format_exponential(value_for_exp, section, exp_token_idx, locale);
    }

    let initial_integer_part_val = adjusted_value.trunc();
    let mut initial_decimal_part = adjusted_value.fract();

    if initial_decimal_part < 0.0 {
        initial_decimal_part = -initial_decimal_part;
    }
    if adjusted_value.fract().abs() < EPSILON || initial_integer_part_val == adjusted_value {
        initial_decimal_part = 0.0;
    }

    let mut decimal_digits_vec: Vec<u8> = Vec::with_capacity(local_decimal_places);
    let mut temp_decimal_part_for_extraction = initial_decimal_part;

    if local_decimal_places > 0 {
        for _ in 0..local_decimal_places {
            temp_decimal_part_for_extraction *= 10.0;
            let digit = temp_decimal_part_for_extraction.trunc() as u8;
            decimal_digits_vec.push(digit.min(9));
            temp_decimal_part_for_extraction -= temp_decimal_part_for_extraction.trunc();
        }
    }
    let final_remaining_decimal = temp_decimal_part_for_extraction.abs();

    let integer_to_format: i64;
    if local_decimal_places == 0 {
        integer_to_format = adjusted_value.round() as i64;
        decimal_digits_vec.clear();
    } else {
        let mut current_integer_part_intermediate = initial_integer_part_val as i64;
        if final_remaining_decimal >= (0.5 - EPSILON) {
            let mut carry = true;
            for i in (0..decimal_digits_vec.len()).rev() {
                if !carry {
                    break;
                }
                decimal_digits_vec[i] += 1;
                if decimal_digits_vec[i] == 10 {
                    decimal_digits_vec[i] = 0;
                    if i == 0 {
                        current_integer_part_intermediate += 1;
                    }
                } else {
                    carry = false;
                }
            }
        }
        integer_to_format = current_integer_part_intermediate;
    }

    let integer_str = integer_to_format.to_string();
    let int_digits: Vec<char> = integer_str.chars().collect();

    // Sign-related flags are now based on `original_value_for_sign`
    let is_negative = original_value_for_sign < 0.0;
    let uses_parentheses = section.tokens.iter().any(|t| {
        matches!(t, FormatToken::LiteralChar('(')) || matches!(t, FormatToken::LiteralChar(')'))
    });

    let should_apply_thousands_separator = section
        .tokens
        .iter()
        .any(|token| matches!(token, FormatToken::ThousandsSeparator));

    let mut formatted_integer_part_vec: Vec<char>;
    if should_apply_thousands_separator && !int_digits.is_empty() && integer_to_format != 0 {
        formatted_integer_part_vec =
            Vec::with_capacity(int_digits.len() + (int_digits.len() - 1) / 3);
        if !(int_digits.len() == 1 && int_digits[0] == '0') {
            for (count, (i, digit)) in int_digits.iter().rev().enumerate().enumerate() {
                if i > 0 && count % 3 == 0 {
                    formatted_integer_part_vec.push(locale.thousands_separator);
                }
                formatted_integer_part_vec.push(*digit);
            }
            formatted_integer_part_vec.reverse();
        } else {
            formatted_integer_part_vec = int_digits.to_vec();
        }
    } else {
        formatted_integer_part_vec = int_digits.to_vec();
    }

    let mut int_digits_iter = formatted_integer_part_vec.iter().cloned().peekable();
    let mut sign_printed = false; // Critical: This tracks if the SECTION's tokens themselves printed a sign.
    let mut in_decimal_part = false;
    let mut frac_pos = 0;

    let mut total_integer_placeholders: usize = 0;
    let mut first_decimal_point_idx: Option<usize> = None;
    for (idx, token) in section.tokens.iter().enumerate() {
        if matches!(token, FormatToken::DecimalPoint) && first_decimal_point_idx.is_none() {
            first_decimal_point_idx = Some(idx);
        }
        if (first_decimal_point_idx.is_none()
            || idx < first_decimal_point_idx.unwrap_or(usize::MAX))
            && matches!(
                token,
                FormatToken::DigitOrZero | FormatToken::DigitIfNeeded | FormatToken::DigitOrSpace
            )
        {
            total_integer_placeholders += 1;
        }
    }

    let num_actual_raw_int_digits = if integer_to_format == 0 && total_integer_placeholders > 0 {
        1
    } else {
        int_digits.len()
    };

    let padding_len = total_integer_placeholders.saturating_sub(num_actual_raw_int_digits);
    let mut current_int_placeholder_idx = 0;
    let mut actual_int_digit_printed = false;

    // Main formatting loop
    for token in &section.tokens {
        match token {
            FormatToken::LiteralChar(c) => {
                let literal_is_acting_as_sign =
                    if !is_positive_section_fallback_for_negative && is_negative {
                        (*c == '(' && uses_parentheses) || (*c == '-' && !uses_parentheses)
                    } else {
                        false
                    };

                // This loop handles cases where integer digits are present but there are no more
                // integer placeholders (e.g. format "#,##0" for value 1234567, the '1' is printed here).
                // OR if there are no integer placeholders at all in the format string.
                while int_digits_iter.peek().is_some()
                    && (current_int_placeholder_idx >= total_integer_placeholders
                        || total_integer_placeholders == 0)
                    && !in_decimal_part
                {
                    result.push(int_digits_iter.next().unwrap());
                    actual_int_digit_printed = true;
                }

                if !sign_printed && literal_is_acting_as_sign {
                    result.push(*c);
                    sign_printed = true;
                } else {
                    result.push(*c);
                }
            }
            FormatToken::QuotedText(text) => {
                while int_digits_iter.peek().is_some()
                    && (current_int_placeholder_idx >= total_integer_placeholders
                        || total_integer_placeholders == 0)
                    && !in_decimal_part
                {
                    result.push(int_digits_iter.next().unwrap());
                    actual_int_digit_printed = true;
                }

                if !sign_printed
                    && !is_positive_section_fallback_for_negative
                    && is_negative
                    && ((!uses_parentheses && text.starts_with('-'))
                        || (uses_parentheses && text.starts_with('(')))
                {
                    sign_printed = true;
                }
                result.push_str(text);
            }
            FormatToken::DecimalPoint => {
                if !actual_int_digit_printed && integer_to_format == 0 {
                    let has_mandatory_int_zero_placeholder = section
                        .tokens
                        .iter()
                        .take_while(|t| !matches!(t, FormatToken::DecimalPoint))
                        .any(|t| matches!(t, FormatToken::DigitOrZero));

                    if has_mandatory_int_zero_placeholder || total_integer_placeholders == 0 {
                        result.push('0');
                        actual_int_digit_printed = true;
                    }
                }
                for digit_char in int_digits_iter.by_ref() {
                    result.push(digit_char);
                    actual_int_digit_printed = true;
                }
                result.push(locale.decimal_point);
                in_decimal_part = true;
            }
            FormatToken::DigitOrZero | FormatToken::DigitIfNeeded | FormatToken::DigitOrSpace => {
                if !in_decimal_part {
                    let mut char_to_print: Option<char> = None;
                    let mut consumed_digit_this_turn = false;
                    if current_int_placeholder_idx < padding_len {
                        match token {
                            FormatToken::DigitOrZero => char_to_print = Some('0'),
                            FormatToken::DigitOrSpace => char_to_print = Some(' '),
                            FormatToken::DigitIfNeeded => {}
                            _ => unreachable!(),
                        }
                    } else if let Some(digit_char_ref) = int_digits_iter.peek() {
                        let digit_char = *digit_char_ref;
                        match token {
                            FormatToken::DigitOrZero | FormatToken::DigitOrSpace => {
                                char_to_print = Some(digit_char);
                                consumed_digit_this_turn = true;
                            }
                            FormatToken::DigitIfNeeded => {
                                if actual_int_digit_printed
                                    || digit_char != '0'
                                    || (num_actual_raw_int_digits == 1 && integer_to_format == 0)
                                    || (integer_to_format == 0
                                        && int_digits_iter.clone().count() == 1)
                                {
                                    char_to_print = Some(digit_char);
                                }
                                consumed_digit_this_turn = true;
                            }
                            _ => unreachable!(),
                        }
                    } else {
                        match token {
                            FormatToken::DigitOrZero => char_to_print = Some('0'),
                            FormatToken::DigitOrSpace => char_to_print = Some(' '),
                            FormatToken::DigitIfNeeded => {}
                            _ => unreachable!(),
                        }
                    }
                    if let Some(p_char) = char_to_print {
                        result.push(p_char);
                        if p_char.is_ascii_digit()
                            && p_char != ' '
                            && (p_char != '0' || matches!(token, FormatToken::DigitOrZero))
                        {
                            actual_int_digit_printed = true;
                        }
                    }
                    if consumed_digit_this_turn {
                        int_digits_iter.next();
                    }
                    current_int_placeholder_idx += 1;
                } else {
                    if frac_pos < decimal_digits_vec.len() {
                        let digit_val = decimal_digits_vec[frac_pos];
                        match token {
                            FormatToken::DigitOrZero | FormatToken::DigitOrSpace => {
                                write!(result, "{}", digit_val).unwrap();
                            }
                            FormatToken::DigitIfNeeded => {
                                let all_subsequent_are_optional_zeros = (frac_pos
                                    ..decimal_digits_vec.len())
                                    .all(|i| decimal_digits_vec[i] == 0);
                                let all_subsequent_placeholders_are_sharp = section
                                    .tokens
                                    .iter()
                                    .skip_while(|t| !matches!(t, FormatToken::DecimalPoint))
                                    .skip(1)
                                    .skip(frac_pos)
                                    .all(|t| !matches!(t, FormatToken::DigitOrZero));
                                if !(digit_val == 0
                                    && all_subsequent_are_optional_zeros
                                    && all_subsequent_placeholders_are_sharp)
                                {
                                    write!(result, "{}", digit_val).unwrap();
                                }
                            }
                            _ => unreachable!(),
                        }
                    } else {
                        match token {
                            FormatToken::DigitOrZero => result.push('0'),
                            FormatToken::DigitOrSpace => result.push(' '),
                            FormatToken::DigitIfNeeded => {}
                            _ => unreachable!(),
                        }
                    }
                    frac_pos += 1;
                }
            }
            FormatToken::Percentage => {
                while int_digits_iter.peek().is_some() {
                    result.push(int_digits_iter.next().unwrap());
                    actual_int_digit_printed = true;
                }
                if !actual_int_digit_printed && integer_to_format == 0 {
                    result.push('0');
                    actual_int_digit_printed = true;
                }
                result.push('%');
            }
            FormatToken::ThousandsSeparator => {}
            FormatToken::TextValue => {
                // This case should ideally not be reached if the early @ check is in place.
                // If it is, it means @ was present but the early exit didn't happen (should not occur).
                // Or, this is a TextValue token in a section that *doesn't* have the overriding @ behavior,
                // which implies a misunderstanding of how TextValue is used outside of the "@ overrides all" rule.
                // For now, treat as a non-event if reached, as primary @ logic is at function start.
            }
            FormatToken::SkipWidth(_) => {
                // As per spec "To create a space that is the width of a character... include an underscore character (_), followed by the character"
                // Simplest implementation is to just add a space. The actual char in SkipWidth(char) is ignored for now.
                result.push(' ');
            }
            // FormatToken::Fill(_): Behavior for fill character is complex and depends on column width, deferred for now.
            _ => {} // Existing catch-all for other tokens like Color, Date/Time (if not handled before or in text mode)
        }
    }

    for digit_char in int_digits_iter {
        result.push(digit_char);
        actual_int_digit_printed = true;
    }

    if !actual_int_digit_printed
        && value_to_format_placeholders == 0.0 // Check against the value used for formatting placeholders
        && result
            .chars()
            .all(|c| c.is_whitespace() || c == '(' || c == ')')
    {
        let has_any_digit_placeholder = section.tokens.iter().any(|t| {
            matches!(
                t,
                FormatToken::DigitOrZero | FormatToken::DigitIfNeeded | FormatToken::DigitOrSpace
            )
        });
        if has_any_digit_placeholder {
            if result.is_empty()
                && has_any_digit_placeholder
                && !section
                    .tokens
                    .iter()
                    .any(|t| matches!(t, FormatToken::DigitOrSpace))
            {
                result.push('0');
            } else if result.trim().is_empty()
                && integer_to_format == 0
                && !has_percentage
                && !after_decimal_flag
                && !is_text_output_mode
            {
                let has_mandatory_zero_placeholder = section
                    .tokens
                    .iter()
                    .any(|t| matches!(t, FormatToken::DigitOrZero));
                if result.is_empty() && has_mandatory_zero_placeholder {
                    result.push('0');
                }
            }
        }
    }

    // Final sign handling, now aware of the context via is_positive_section_fallback_for_negative
    if is_negative {
        // is_negative is based on original_value_for_sign
        if uses_parentheses {
            // Parentheses are usually part of an explicit negative (or sometimes positive) section design.
            // `sign_printed` should be true if the section's tokens included '('.
            if !sign_printed {
                result.insert(0, '(');
            }
            if result.starts_with('(') && !result.ends_with(')') {
                result.push(')');
            }
        } else {
            // Not using parentheses for negative sign
            if is_positive_section_fallback_for_negative {
                // Excel behavior for P(abs(V)): prepend '-' to the result of (PositiveSection applied to Abs(Value)).
                // This happens UNCONDITIONALLY when it's a fallback for a negative value,
                // as the positive section's literals are part of P(abs(V)).
                result.insert(0, '-');
            } else if !sign_printed {
                // This is for an actual negative section that was chosen, but it doesn't use parentheses
                // AND its tokens did not include a literal '-' to set sign_printed = true.
                // E.g., format `0.0; [Red]0.0` -> negative section is `[Red]0.0`, sign_printed would be false.
                result.insert(0, '-');
            }
            // If not is_positive_section_fallback_for_negative AND sign_printed is true,
            // it means an explicit negative section (e.g. `0.0; "-"0.0`) with its own literal sign was used and handled it.
        }
    }
    result
}
