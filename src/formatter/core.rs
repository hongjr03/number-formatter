use crate::formatter::exponential;
use crate::types::{FormatSection, FormatToken, LocaleSettings};
use std::fmt::Write;

/// Format a numeric value using the specified format section
pub(super) fn format_value(value: f64, section: &FormatSection, locale: &LocaleSettings) -> String {
    const EPSILON: f64 = 1e-9; // Define EPSILON for rounding
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
    let mut adjusted_value = if has_percentage {
        abs_value * 100.0
    } else {
        abs_value
    };

    // Apply scaling based on num_scaling_commas
    if section.num_scaling_commas > 0 {
        for _ in 0..section.num_scaling_commas {
            adjusted_value /= 1000.0;
        }
    }

    // Determine decimal_places from format tokens
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

    // Step 3: Handle exponential notation if present
    // This comes after scaling/percentage, but before other numeric formatting/rounding.
    if let Some(exp_token_idx) = section
        .tokens
        .iter()
        .position(|t| matches!(t, FormatToken::Exponential(_)))
    {
        let value_for_exp = if value < 0.0 {
            -adjusted_value // Apply sign back to the adjusted value
        } else {
            adjusted_value
        };
        return exponential::format_exponential(value_for_exp, section, exp_token_idx, locale);
    }

    // Step 4: Separate initial integer and decimal parts
    let initial_integer_part_val = adjusted_value.trunc();
    let mut initial_decimal_part = adjusted_value.fract();

    // Ensure initial_decimal_part is non-negative for consistent digit extraction
    if initial_decimal_part < 0.0 {
        initial_decimal_part = -initial_decimal_part;
    }
    // Correct for very small negative fracs that might occur from positive adjusted_value due to precision
    if adjusted_value >= 0.0
        && initial_decimal_part > (1.0 - EPSILON)
        && initial_decimal_part != 0.0
    {
        // e.g. 2.0.fract() could be -0.0 or very close to 1.0 from negative side
        if adjusted_value.fract().abs() < EPSILON {
            // if original fract was tiny
            initial_decimal_part = 0.0;
        } else if initial_integer_part_val == adjusted_value {
            // e.g. value is 2.0, adjusted_value.fract() is 0.0
            initial_decimal_part = 0.0;
        }
        // If initial_decimal_part was like 0.99999999999 due to adjusted_value = 1.99999999999 -> 2.0.trunc() = 2, .fract() = -small_val
        // this part needs to be careful. For adjusted_value = 1.999999999999, trunc is 1.0, fract is 0.999999999999
    }

    // Step 5: Extract decimal digits and calculate final_remaining_decimal
    let mut decimal_digits_vec: Vec<u8> = Vec::with_capacity(local_decimal_places);
    let mut temp_decimal_part_for_extraction = initial_decimal_part;

    if local_decimal_places > 0 {
        for _ in 0..local_decimal_places {
            temp_decimal_part_for_extraction *= 10.0;
            let digit = temp_decimal_part_for_extraction.trunc() as u8;
            decimal_digits_vec.push(digit.min(9)); // Ensure digit is not > 9 due to precision
            temp_decimal_part_for_extraction -= temp_decimal_part_for_extraction.trunc(); // Use .trunc() to remove integer part
        }
    }
    // final_remaining_decimal is what's left of temp_decimal_part_for_extraction
    // ensure it's positive for rounding comparison.
    let final_remaining_decimal = temp_decimal_part_for_extraction.abs();

    // Step 6: Rounding Decision and Integer Update
    let integer_to_format: i64;

    if local_decimal_places == 0 {
        integer_to_format = adjusted_value.round() as i64;
        decimal_digits_vec.clear(); // Ensure it's empty
    } else {
        // Start with the integer part derived from truncating the (scaled/percentaged) adjusted_value
        let mut current_integer_part_intermediate = initial_integer_part_val as i64;

        if final_remaining_decimal >= (0.5 - EPSILON) {
            let mut carry = true; // Start with a carry for the rounding
            for i in (0..decimal_digits_vec.len()).rev() {
                if !carry {
                    break;
                }
                decimal_digits_vec[i] += 1;
                if decimal_digits_vec[i] == 10 {
                    decimal_digits_vec[i] = 0;
                    // carry remains true
                    if i == 0 {
                        // Carry out of the most significant decimal place
                        current_integer_part_intermediate += 1;
                    }
                } else {
                    carry = false; // No more carry needed
                }
            }
        }
        integer_to_format = current_integer_part_intermediate;
    }

    // Step 7: Generate int_digits string
    // Note: integer_to_format can be 0 if value was e.g. 0.005, format "0", rounds to 0.
    // or -0.005, format "0", rounds to 0.
    // if value was 0.7, format "0", rounds to 1. adjusted_value is 0.7, integer_to_format is 1.
    // if value was -0.7, format "0", rounds to -1. adjusted_value is 0.7, initial_integer_part_val is 0.
    // rounding for local_decimal_places == 0 should handle sign correctly via adjusted_value.round().
    // For local_decimal_places > 0, integer_to_format gets its sign from initial_integer_part_val.
    // If value was -0.7, adjusted_value = 0.7, initial_integer_part_val = 0. final_remaining_decimal = 0.7.
    // current_integer_part_intermediate becomes 0. decimal_digits_vec might become [7].
    // If rounding 0.7 to 0dp (handled by first branch), integer_to_format = 1.
    // If -0.7 to 0dp, adjusted_value = 0.7, integer_to_format = 0.7.round() = 1. This is for magnitude. Sign is handled later.
    // The integer_to_format should be the absolute magnitude. Sign applied at the end.

    // If integer_to_format ended up negative from rounding (e.g. -0.5.round() -> -1 in some contexts, but f64::round ties to even, so -0.5.round() is 0.0 )
    // We are operating on abs_value for adjusted_value, so integer_to_format will be non-negative here.
    // is_negative flag (from original value) will be used for final sign.

    let integer_str = integer_to_format.to_string(); // This will be non-negative
    let int_digits: Vec<char> = integer_str.chars().collect();

    // 基础值处理
    let is_negative = value < 0.0;
    let uses_parentheses = section.tokens.iter().any(|t| {
        matches!(t, FormatToken::LiteralChar('(')) || matches!(t, FormatToken::LiteralChar(')'))
    });

    // Determine if thousands separators should be applied for this section
    let should_apply_thousands_separator = section
        .tokens
        .iter()
        .any(|token| matches!(token, FormatToken::ThousandsSeparator));

    let mut formatted_integer_part_vec: Vec<char>;
    if should_apply_thousands_separator && !int_digits.is_empty() && integer_to_format != 0 {
        // also check if integer_to_format is not 0
        formatted_integer_part_vec =
            Vec::with_capacity(int_digits.len() + (int_digits.len() - 1) / 3);
        // If int_digits is ["0"], no separator.
        if !(int_digits.len() == 1 && int_digits[0] == '0') {
            for (count, (i, digit)) in int_digits.iter().rev().enumerate().enumerate() {
                if i > 0 && count % 3 == 0 {
                    formatted_integer_part_vec.push(locale.thousands_separator);
                }
                formatted_integer_part_vec.push(*digit);
            }
            formatted_integer_part_vec.reverse(); // Reverse back to correct order
        } else {
            formatted_integer_part_vec = int_digits.to_vec(); // Keep ["0"] as is
        }
    } else {
        formatted_integer_part_vec = int_digits.to_vec(); // Use original digits if no separator
    }

    let mut int_digits_iter = formatted_integer_part_vec.iter().cloned().peekable();
    let mut sign_printed = false;
    let mut in_decimal_part = false;
    let mut frac_pos = 0; // For indexing decimal_digits_vec

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
        1 // Consider "0" as having 1 digit for padding if there are placeholders
    } else {
        int_digits.len() // Raw length of the number string "0", "123", etc.
    };

    let padding_len = total_integer_placeholders.saturating_sub(num_actual_raw_int_digits);
    let mut current_int_placeholder_idx = 0;
    let mut actual_int_digit_printed = false;

    // Main formatting loop
    for token in &section.tokens {
        match token {
            FormatToken::LiteralChar(c) => {
                let is_potential_open_paren_sign = *c == '(' && is_negative && uses_parentheses;
                let is_potential_dash_sign = *c == '-' && is_negative && !uses_parentheses;

                // Drain remaining integer digits if placeholders are exhausted before this literal
                while int_digits_iter.peek().is_some()
                    && (current_int_placeholder_idx >= total_integer_placeholders
                        || total_integer_placeholders == 0)
                    && !in_decimal_part
                {
                    if !sign_printed && is_negative && !uses_parentheses {
                        // This condition applies for prefixing '-' before digits
                        result.push('-');
                        sign_printed = true;
                    }
                    result.push(int_digits_iter.next().unwrap());
                    actual_int_digit_printed = true;
                }

                // Handle printing of the literal character itself, considering if it acts as a sign
                if !sign_printed {
                    if is_potential_open_paren_sign {
                        result.push('('); // Print the char that is the opening parenthesis sign
                        sign_printed = true;
                    } else if is_potential_dash_sign {
                        result.push('-'); // Print the char that is the dash sign
                        sign_printed = true;
                    } else {
                        // This literal is not acting as a sign, or sign is already printed.
                        // Or it could be a sign char but for a positive number, or wrong context.
                        result.push(*c); // Print a non-sign literal or a sign literal in non-signing context
                    }
                } else {
                    // Sign has already been printed by a previous token or mechanism.
                    // So, just print the current literal character as is.
                    // This covers cases like format "--#" for a negative value: first '-' prints sign, second '-' is literal.
                    // Or "((#))" for negative: first '(' prints sign, second '(' is literal.
                    result.push(*c);
                }
            }
            FormatToken::QuotedText(text) => {
                // Drain remaining integer digits if placeholders are exhausted
                while int_digits_iter.peek().is_some()
                    && (current_int_placeholder_idx >= total_integer_placeholders
                        || total_integer_placeholders == 0)
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
                if !actual_int_digit_printed && integer_to_format == 0 {
                    let has_mandatory_int_zero_placeholder = section
                        .tokens
                        .iter()
                        .take_while(|t| !matches!(t, FormatToken::DecimalPoint))
                        .any(|t| matches!(t, FormatToken::DigitOrZero));

                    if !sign_printed && is_negative && !uses_parentheses {
                        // Moved sign printing before potential '0'
                        result.push('-');
                        sign_printed = true;
                    }
                    if has_mandatory_int_zero_placeholder || total_integer_placeholders == 0 {
                        result.push('0');
                        actual_int_digit_printed = true;
                    }
                }

                for digit_char in int_digits_iter.by_ref() {
                    if !sign_printed && is_negative && !uses_parentheses {
                        // Sign for digits before decimal point if not yet printed
                        result.push('-');
                        sign_printed = true;
                    }
                    result.push(digit_char);
                    actual_int_digit_printed = true;
                }
                result.push(locale.decimal_point);
                in_decimal_part = true;
            }
            FormatToken::DigitOrZero | FormatToken::DigitIfNeeded | FormatToken::DigitOrSpace => {
                if !in_decimal_part {
                    // Integer part placeholder
                    if !sign_printed && is_negative && !uses_parentheses && result.is_empty() {
                        // Only prefix sign if no other content (like literal prefix) is present
                        // And if we are expecting to print a digit now
                        let is_first_substantive_char_opportunity =
                            result.chars().all(|c| c.is_whitespace() || c == '(');
                        if is_first_substantive_char_opportunity {
                            result.push('-');
                            sign_printed = true;
                        }
                    }

                    let mut char_to_print: Option<char> = None;
                    let mut consumed_digit_this_turn = false;

                    if current_int_placeholder_idx < padding_len {
                        // In padding region
                        match token {
                            FormatToken::DigitOrZero => char_to_print = Some('0'),
                            FormatToken::DigitOrSpace => char_to_print = Some(' '),
                            FormatToken::DigitIfNeeded => {} // Print nothing for '#' in padding
                            _ => unreachable!(),
                        }
                    } else if let Some(digit_char_ref) = int_digits_iter.peek() {
                        // Actual digit available
                        let digit_char = *digit_char_ref;
                        // Logic for printing the digit or not based on placeholder type
                        match token {
                            FormatToken::DigitOrZero | FormatToken::DigitOrSpace => {
                                char_to_print = Some(digit_char);
                                consumed_digit_this_turn = true;
                            }
                            FormatToken::DigitIfNeeded => {
                                // Print if it's not a leading zero (unless it's the only digit and is zero)
                                // actual_int_digit_printed tracks if a *significant* digit has been printed
                                if actual_int_digit_printed
                                    || digit_char != '0'
                                    || (num_actual_raw_int_digits == 1 && integer_to_format == 0)
                                {
                                    char_to_print = Some(digit_char);
                                } else if !actual_int_digit_printed
                                    && digit_char == '0'
                                    && int_digits_iter.clone().count() == 1
                                    && integer_to_format == 0
                                {
                                    // Case: value is 0, format is "#" or "##". We should print one "0" if it's the only digit for the integer part.
                                    // This can be tricky. Let's simplify: if it's a '0', and no actual_int_digit_printed, and it's the last int digit, print.
                                    // The original condition: actual_int_digit_printed || digit_char != '0' || (num_actual_raw_int_digits == 1 && integer_part == 0)
                                    // Here integer_part is integer_to_format.
                                    char_to_print = Some(digit_char);
                                }
                                consumed_digit_this_turn = true;
                            }
                            _ => unreachable!(),
                        }
                    } else {
                        // No more actual digits, but still in placeholders (e.g. format "000", value 1)
                        match token {
                            FormatToken::DigitOrZero => char_to_print = Some('0'),
                            FormatToken::DigitOrSpace => char_to_print = Some(' '),
                            FormatToken::DigitIfNeeded => {} // Print nothing for '#' if no digit
                            _ => unreachable!(),
                        }
                    }

                    if let Some(p_char) = char_to_print {
                        if !sign_printed
                            && is_negative
                            && !uses_parentheses
                            && result.is_empty()
                            && p_char != ' '
                        {
                            result.push('-');
                            sign_printed = true;
                        }
                        result.push(p_char);
                        if p_char.is_ascii_digit() && p_char != ' ' {
                            // Consider any printed digit as "actual"
                            // Specifically, if we print a '0' due to DigitOrZero placeholder, it counts.
                            if p_char != '0' || matches!(token, FormatToken::DigitOrZero) {
                                actual_int_digit_printed = true;
                            }
                        }
                    }

                    if consumed_digit_this_turn {
                        int_digits_iter.next();
                    }
                    current_int_placeholder_idx += 1;
                } else {
                    // Decimal part placeholder
                    if frac_pos < decimal_digits_vec.len() {
                        let digit_val = decimal_digits_vec[frac_pos];
                        match token {
                            FormatToken::DigitOrZero | FormatToken::DigitOrSpace => {
                                write!(result, "{}", digit_val).unwrap();
                            }
                            FormatToken::DigitIfNeeded => {
                                // Print if digit is non-zero, or if it's a zero that's not a trailing zero
                                // (unless all subsequent requested digits are also zero)
                                let is_trailing_zero = digit_val == 0
                                    && (frac_pos..decimal_digits_vec.len())
                                        .all(|i| decimal_digits_vec[i] == 0);
                                // And if format requests only up to this zero.
                                // Count remaining '#' or '0' placeholders in format string for decimal part
                                let remaining_required_placeholders = section
                                    .tokens
                                    .iter()
                                    .skip_while(|t| !matches!(t, FormatToken::DecimalPoint))
                                    .skip(1) //
                                    .skip(frac_pos + 1)
                                    .filter(|t| {
                                        matches!(
                                            t,
                                            FormatToken::DigitOrZero | FormatToken::DigitIfNeeded
                                        )
                                    })
                                    .count();

                                if !is_trailing_zero || remaining_required_placeholders > 0 {
                                    write!(result, "{}", digit_val).unwrap();
                                } else if digit_val == 0
                                    && local_decimal_places > 0
                                    && frac_pos == 0
                                    && decimal_digits_vec.iter().all(|&d| d == 0)
                                    && matches!(token, FormatToken::DigitIfNeeded)
                                {
                                    // Special case for "0.0" format "#.##" value 0 -> "0". Here, we don't print the zero.
                                    // But if format "0.00", value 0 -> "0.00". Our current vec has [0,0].
                                    // This part means: if format is #, and digit is 0, and it's a trailing zero, print nothing.
                                    // This seems to conflict with excel: 0 format "#.##" -> "0."
                                    // Let's use a simpler rule for now: print if DigitOrZero, or if DigitIfNeeded and not a trailing optional zero.
                                    // A zero is "trailing optional" if it's a 0, and all subsequent digits in decimal_digits_vec are 0,
                                    // AND all subsequent placeholders are DigitIfNeeded.
                                    let all_subsequent_are_optional_zeros = (frac_pos
                                        ..decimal_digits_vec.len())
                                        .all(|i| decimal_digits_vec[i] == 0);
                                    let all_subsequent_placeholders_are_sharp = section
                                        .tokens
                                        .iter()
                                        .skip_while(|t| !matches!(t, FormatToken::DecimalPoint))
                                        .skip(1)
                                        .skip(frac_pos) // Start from current placeholder
                                        .all(|t| !matches!(t, FormatToken::DigitOrZero)); // No '0' placeholders after this

                                    if !(digit_val == 0
                                        && all_subsequent_are_optional_zeros
                                        && all_subsequent_placeholders_are_sharp)
                                    {
                                        write!(result, "{}", digit_val).unwrap();
                                    }
                                }
                            }
                            _ => unreachable!(),
                        }
                    } else {
                        // No more digits in decimal_digits_vec, but still in decimal placeholders
                        match token {
                            FormatToken::DigitOrZero => result.push('0'),
                            FormatToken::DigitOrSpace => result.push(' '),
                            FormatToken::DigitIfNeeded => {} // Print nothing
                            _ => unreachable!(),
                        }
                    }
                    frac_pos += 1;
                }
            }
            FormatToken::Percentage => {
                // Drain remaining integer digits
                while int_digits_iter.peek().is_some() {
                    if !sign_printed && is_negative && !uses_parentheses && result.is_empty() {
                        result.push('-');
                        sign_printed = true;
                    }
                    result.push(int_digits_iter.next().unwrap());
                    actual_int_digit_printed = true;
                }
                if !actual_int_digit_printed && integer_to_format == 0 {
                    // e.g. value 0, format "0%"
                    if !sign_printed && is_negative && !uses_parentheses && result.is_empty() {
                        result.push('-');
                        sign_printed = true;
                    }
                    result.push('0');
                    actual_int_digit_printed = true;
                }
                result.push('%');
            }
            FormatToken::ThousandsSeparator => {
                // Thousands separators are handled by pre-formatting formatted_integer_part_vec.
                // This token existing in the list primarily serves as a flag for `should_apply_thousands_separator`.
                // However, if a comma appears *not* where a separator would go, it might be a literal.
                // For now, we assume its presence implies the formatting rule.
                // A sign might need to be printed if it hasn't and this is the first effective char.
                if !sign_printed && is_negative && !uses_parentheses && result.is_empty() {
                    // If a comma from format string is the very first thing, and we are negative.
                    // This is unlikely for a thousands separator token itself, but for robustness.
                    // result.push('-');
                    // sign_printed = true;
                }
            }
            FormatToken::TextValue => { // Handle @ token - insert value as text
                // This is complex: if value is number, format as general number, then insert.
                // If value is text, insert text. This function currently assumes numeric input.
                // For now, let's assume if @ is hit, we format `value` generally (needs a sub-format or default)
                // and insert it. This is a placeholder for a more robust feature.
                // Simplification: if @ is present, it's likely part of a text section, which `is_text_output_mode` should handle.
                // If it's in a numeric section for a number, its behavior is to format the number as if it were text.
                // This would mean `value.to_string()` or a simple default.
                // Let's defer full @ implementation for numbers here.
            }
            // Other tokens like Fill, SkipWidth, Color are mostly for structure/ignored in this direct formatting pass
            _ => {}
        }
    }

    // Drain any remaining int_digits that were not consumed by placeholders (e.g. format "#", value 123)
    for digit_char in int_digits_iter {
        if !sign_printed && is_negative && !uses_parentheses && result.is_empty() {
            result.push('-');
            sign_printed = true;
        }
        result.push(digit_char);
        actual_int_digit_printed = true;
    }

    // If nothing substantial was printed (e.g. format "###" for value 0, or format "" for 0)
    // and value is 0, print "0" unless it's a purely text format (which is handled by is_text_output_mode)
    // or if the format specifically was meant to produce empty for zero (e.g. some conditional formats).
    if !actual_int_digit_printed
        && value == 0.0
        && result
            .chars()
            .all(|c| c.is_whitespace() || c == '(' || c == ')')
    {
        // Check if the format string implies that zero should be blank or if it should be "0"
        let has_any_digit_placeholder = section.tokens.iter().any(|t| {
            matches!(
                t,
                FormatToken::DigitOrZero | FormatToken::DigitIfNeeded | FormatToken::DigitOrSpace
            )
        });
        if has_any_digit_placeholder {
            // if format was like "###" or "000" or "   "
            if !sign_printed && is_negative && !uses_parentheses { // for -0.0
                // result.push('-'); // Sign might be inside parentheses
                // sign_printed = true;
            }
            // If all placeholders were spaces or optional, result might be empty or spaces.
            // If at least one '0' placeholder existed, it should have been printed.
            // If only '#' or ' ' and value is 0, then `actual_int_digit_printed` would be false.
            // Excel behavior for 0 with format "#" is empty. For "0" is "0". For "?.?" is " . ".
            // This condition is to ensure *something* is printed if it should be.
            // If result is truly empty (not even spaces from '?' placeholders), and there was some numeric placeholder, print '0'.
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
                // If result is effectively empty (only spaces), and it was a zero value, not percentage, not explicitly decimal.
                // This is to catch cases like format "???" value 0 -> "   " is fine.
                // format "#" value 0 -> "" is fine.
                // format "" value 0 -> "" is fine.
                // format "General" value 0 -> "0"
                // This needs to be careful not to override intentional empty strings for zero.
                // Let's assume if actual_int_digit_printed is false for a zero value, and result is empty,
                // AND there was at least one "0" placeholder, it means "0" should have been printed by placeholder logic.
                // If only "#" or "?", it's ok for it to be empty or spaces.
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

    // Final sign handling
    if is_negative {
        if uses_parentheses {
            if !sign_printed {
                // If '(' was not part of the format string and printed
                result.insert(0, '(');
            }
            // Ensure ')' is there if '(' was.
            // If result does not end with ')' but starts with '(', add it.
            // This is a bit simplified. Proper parenthesizing depends on format structure.
            if result.starts_with('(') && !result.ends_with(')') {
                result.push(')');
            } else if !result.starts_with('(') && sign_printed { // sign_printed true means '(' was from format.
                // This case implies format had '(' but maybe not ')'. This is unusual.
            }
        } else if !sign_printed {
            // If no sign has been printed by any other logic (e.g. before first digit, or by literal '-')
            // Prepend a standard minus sign.
            // This should only happen if the format string itself doesn't dictate sign position.
            let mut insertion_point = 0;
            for (i, char_code) in result.char_indices() {
                if char_code.is_ascii_digit() {
                    // insert before first digit
                    insertion_point = i;
                    break;
                } else if char_code != ' ' {
                    // insert before first non-space if no digits (e.g. "$.--")
                    insertion_point = i;
                    break;
                }
                if i == result.len() - 1 {
                    // if all spaces, insert at end (becomes beginning after rev)
                    insertion_point = result.len();
                }
            }
            // A simpler approach: if no sign printed, and it's negative, just prepend if not parenthesized.
            // However, consider formats like "$ #.##" for -$10 -> "$-10.00" vs "-$10.00"
            // Standard is usually -$10.00 or ($10.00).
            // For now, a simple prepend if no sign character is found.
            if !result.contains('-') && !result.contains('(') {
                // Check again as sign_printed could be misleading
                result.insert(0, '-');
            }
        }
    }
    result
}
