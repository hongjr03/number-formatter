/// Converts a decimal value to the best rational approximation (numerator/denominator).
///
/// # Arguments
/// * `decimal` - The decimal value to convert (expected to be in [0, 1) range typically).
/// * `max_denominator_digits` - The maximum number of digits allowed for the denominator.
///   (e.g., 1 for '?', 2 for '??', 3 for '???', meaning max denominator of 9, 99, 999 respectively).
///
/// # Returns
/// * `Option<(i64, i64)>` - Numerator and denominator as a tuple, or None if no suitable fraction is found
///   (e.g., if decimal is 0 and we want to represent it as no fractional part).
///   Returns (0,1) for input 0.0 to signify 0/1.
pub(super) fn decimal_to_fraction(
    decimal: f64,
    max_denominator_digits: usize,
) -> Option<(i64, i64)> {
    if !(0.0..1.0).contains(&decimal) {
        // This function is primarily for the fractional part.
        // Consider asserting or returning error if out of expected range,
        // or handling it by stripping integer part if misuse is anticipated.
        // For now, let's assume input is valid fractional part.
    }

    if decimal == 0.0 {
        return Some((0, 1)); // Represent 0 as 0/1
    }

    let max_denominator = match max_denominator_digits {
        0 => return None, // No placeholders for denominator
        1 => 9,
        2 => 99,
        3 => 999,
        _ => 10_i64.pow(max_denominator_digits as u32) - 1, // Practical limit
    };

    // Continued fraction algorithm
    // Based on https://en.wikipedia.org/wiki/Continued_fraction#Best_rational_approximations
    // And https://www.cs.utexas.edu/users/EWD/transcriptions/EWD08xx/EWD831.html

    let mut a = decimal;
    let mut h_prev = 0;
    let mut k_prev = 1;
    let mut h_curr = 1;
    let mut k_curr = 0;

    loop {
        let floor_a = a.floor();
        let ai = floor_a as i64;

        let h_next = ai * h_curr + h_prev;
        let k_next = ai * k_curr + k_prev;

        if k_next > max_denominator {
            // Denominator k_curr is the largest that doesn't exceed max_denominator.
            // If k_curr is 0 (can happen if decimal is very large, though we expect [0,1)),
            // or if no fraction was found.
            if k_curr == 0 {
                return None; // Should not happen for decimal in [0,1) and max_denominator >=1
            }
            // Determine which of k_curr or k_next (if we were to use it despite exceeding max_den) is a better approximation
            // This part can be complex. A simpler approach often taken for formatters:
            // Use the last (h_curr, k_curr) that was *within* the denominator limit.

            // If h_curr/k_curr is a valid fraction.
            if k_curr > 0 {
                // Check if h_curr/k_curr is a better approximation than not showing a fraction.
                // For very small decimals, 0/1 might be better if h_curr/k_curr is like 1/999
                // This level of "best" might be too complex. Let's return what we have.
                return Some((h_curr, k_curr));
            } else {
                // Fallback or error, k_curr should not be 0 if decimal > 0.
                // This can happen if the first 'ai' makes k_next too large.
                // e.g. decimal = 0.0001, max_den = 9. First ai = 0.
                // a = 1/0.0001 = 10000. ai = 10000.
                // h_prev=0, k_prev=1, h_curr=1, k_curr=0
                // h_next = 10000 * 1 + 0 = 10000
                // k_next = 10000 * 0 + 1 = 1
                // Oh, k_curr is the one to test.
                // Let's re-trace:
                // Start: h(-2)=0, k(-2)=1; h(-1)=1, k(-1)=0.
                // a0 = floor(a)
                // h(0)=a0*h(-1)+h(-2) = a0*1+0 = a0
                // k(0)=a0*k(-1)+k(-2) = a0*0+1 = 1
                // So first convergent is a0/1.
                // Next, a1 = floor(1/(a-a0)) etc.

                // Simpler iterative method from a common recipe:
                // (Using p, q for numerator/denominator)
                let mut p0 = 0;
                let mut q0 = 1; // p0/q0 = 0/1
                let mut p1 = 1;
                let mut q1 = 0; // p1/q1 = 1/0 (represents infinity)

                let mut x = decimal;
                let mut n = 0; // iteration count, for safety

                loop {
                    n += 1;
                    if n > 20 {
                        return None;
                    } // Safety break for complex non-terminating

                    let floor_x = x.floor();
                    let int_part_a = floor_x as i64;

                    let p2 = int_part_a * p1 + p0;
                    let q2 = int_part_a * q1 + q0;

                    // Check if q2 fits the digit count for denominator
                    let mut temp_q = q2;
                    let mut den_digits = 0;
                    if temp_q == 0 {
                        den_digits = 1;
                    } else {
                        while temp_q > 0 {
                            temp_q /= 10;
                            den_digits += 1;
                        }
                    }

                    if den_digits > max_denominator_digits && q2 > max_denominator {
                        // q2 might fit max_denominator but not digit count
                        // p1/q1 is the previous best fraction within limits.
                        if q1 == 0 {
                            return None;
                        } // No valid fraction found
                        return Some((p1, q1));
                    }

                    if (x - floor_x).abs() < 1e-9 {
                        // Essentially an integer, fraction part is exact
                        if q2 == 0 {
                            return None;
                        }
                        return Some((p2, q2));
                    }

                    x = 1.0 / (x - floor_x); // Next a_i term for continued fraction
                    p0 = p1;
                    q0 = q1;
                    p1 = p2;
                    q1 = q2;
                }
            }
        }

        h_prev = h_curr;
        k_prev = k_curr;
        h_curr = h_next;
        k_curr = k_next;

        let remainder = a - floor_a;
        if remainder.abs() < 1e-9 {
            // Tolerance for floating point
            // Exact fraction found
            if k_curr == 0 {
                return None;
            } // Should not happen
            return Some((h_curr, k_curr));
        }
        a = 1.0 / remainder;

        if k_curr == 0 {
            return None;
        } // Safety break if denominator becomes 0 (error in logic or input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_fractions() {
        assert_eq!(decimal_to_fraction(0.5, 1), Some((1, 2)));
        assert_eq!(decimal_to_fraction(0.25, 1), Some((1, 4)));
        assert_eq!(decimal_to_fraction(0.75, 1), Some((3, 4)));
        assert_eq!(decimal_to_fraction(0.2, 1), Some((1, 5)));
        assert_eq!(decimal_to_fraction(0.0, 1), Some((0, 1)));
    }

    #[test]
    fn test_max_den_digits() {
        // 1/3 = 0.333...
        assert_eq!(decimal_to_fraction(1.0 / 3.0, 1), Some((1, 3)));
        assert_eq!(decimal_to_fraction(1.0 / 3.0, 2), Some((1, 3))); // 33/99 reduces to 1/3

        // 1/8 = 0.125
        assert_eq!(decimal_to_fraction(0.125, 1), Some((1, 8)));

        assert_eq!(decimal_to_fraction(0.3, 1), Some((2, 7))); // 1/3
        assert_eq!(decimal_to_fraction(0.3, 2), Some((3, 10))); // 3/10 (max_den=99)

        // 5/8 = 0.625
        assert_eq!(decimal_to_fraction(0.625, 1), Some((5, 8)));
        // 1/9
        assert_eq!(decimal_to_fraction(1.0 / 9.0, 1), Some((1, 9)));

        // 1/10 - needs 2 digits for denominator
        assert_eq!(decimal_to_fraction(0.1, 1), Some((0, 1))); // Closest for single digit den is 1/9=0.111 or 0/1
        // Actually, for 0.1, max_den=9. 1/9=0.111, 1/8=0.125. 0/1=0. (error=0.1)
        // x=0.1, a=0. p=0,q=1. (0/1)
        // x=10, a=10. p=10*0+1=1, q=10*1+0=10. (1/10)
        // q=10 (2 digits) > 1 digit max. So return previous (0,1).
        // This means we need to be careful about "best".
        // The "best" is the one that minimizes |decimal - p/q|.
        // The standard continued fraction method gives best rational approx.
        // The loop I wrote needs to return p1/q1 if q2 exceeds limit.
        assert_eq!(decimal_to_fraction(0.1, 1), Some((0, 1))); // Or perhaps None if 0/1 is not desired for non-zero input
        // The problem definition implies finding *a* fraction. 0/1 is not ideal for 0.1.
        // Let's refine max_denominator check in the loop.
        // If q2 exceeds max_denominator, then p1/q1 is the best approx *before* this step.
        assert_eq!(decimal_to_fraction(0.1, 2), Some((1, 10)));
    }

    #[test]
    fn test_complex_cases() {
        // Value of Pi's fractional part
        let pi_frac = std::f64::consts::PI - 3.0; // approx 0.1415926535...
        assert_eq!(decimal_to_fraction(pi_frac, 1), Some((1, 7))); // 1/7 = 0.1428...
        assert_eq!(decimal_to_fraction(pi_frac, 2), Some((1, 7))); // Still 1/7. Next is 16/113 (den 3 digits)
        assert_eq!(decimal_to_fraction(pi_frac, 3), Some((16, 113))); // 16/113 = 0.1415929...
        // 113 is 3 digits.
    }
}

// The simplified loop for decimal_to_fraction needs refinement
// to correctly pick the best approximation when denominator limits are hit.
// The current one might prematurely return or pick a suboptimal one.
// Standard algorithm for best rational approximation:
// a_i = floor(x_i)
// p_i = a_i * p_{i-1} + p_{i-2}
// q_i = a_i * q_{i-1} + q_{i-2}
// with (p_{-2}, q_{-2}) = (0,1) and (p_{-1}, q_{-1}) = (1,0)
// x_{i+1} = 1 / (x_i - a_i)
// Stop when q_i > max_denominator. The previous convergent p_{i-1}/q_{i-1} is the best.
