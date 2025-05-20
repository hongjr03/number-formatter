use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

/// Helper function to convert f64 Excel date to NaiveDateTime
/// Takes into account Excel's leap year bug (1900-02-29 is valid)
pub fn convert_f64_to_datetime(value: f64) -> Option<NaiveDateTime> {
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
