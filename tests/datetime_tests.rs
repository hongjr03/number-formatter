use number_format::formatter::format_number;
use number_format::parser::parse_number_format;
use number_format::types::LocaleSettings;

// Helper to create default locale settings
fn default_loc() -> LocaleSettings {
    LocaleSettings::default()
}

// Helper for custom locale (e.g., a simplified French example)
fn french_loc() -> LocaleSettings {
    LocaleSettings::default()
        .with_ampm_markers(["mat.", "apr.m."]) // Fictional short AM/PM for French
        .with_short_day_names(["Dim", "Lun", "Mar", "Mer", "Jeu", "Ven", "Sam"])
        .with_day_names([
            "Dimanche", "Lundi", "Mardi", "Mercredi", "Jeudi", "Vendredi", "Samedi",
        ])
        .with_short_month_names([
            "janv.", "févr.", "mars", "avr.", "mai", "juin", "juil.", "août", "sept.", "oct.",
            "nov.", "déc.",
        ])
        .with_month_names([
            "janvier",
            "février",
            "mars",
            "avril",
            "mai",
            "juin",
            "juillet",
            "août",
            "septembre",
            "octobre",
            "novembre",
            "décembre",
        ])
    // Assuming default decimal and thousands separators are fine for this French example for now,
    // otherwise, they could be set here too: e.g., .with_decimal_point(',').with_thousands_separator(' ')
}

const TEST_DATE_SERIAL: f64 = 45292.75; // Represents 2024-01-01 18:00:00 as per user input for date part
const TEST_DATE_SERIAL_MORNING: f64 = 45292.375; // 2024-01-01 09:00:00 (Monday AM)
const TEST_DATE_NEAR_MIDNIGHT: f64 = 45292.9999884259; // 2024-01-01 23:59:59
const TEST_DURATION_SERIAL: f64 = 1.5432175925925926; // 1 day, 12h, 33m, 25s, .5ms

// --- Basic Date Tests ---
#[test]
fn test_date_yyyy_mm_dd() {
    let fmt = parse_number_format("yyyy-mm-dd").unwrap();
    assert_eq!(
        format_number(TEST_DATE_SERIAL, &fmt, &default_loc()),
        "2024-01-01"
    );
}

#[test]
fn test_date_yy_m_d() {
    let fmt = parse_number_format("yy-m-d").unwrap();
    assert_eq!(
        format_number(TEST_DATE_SERIAL, &fmt, &default_loc()),
        "24-1-1"
    );
}

// --- Month Representations ---
#[test]
fn test_month_formats() {
    let val = TEST_DATE_SERIAL; // January
    assert_eq!(
        format_number(val, &parse_number_format("m").unwrap(), &default_loc()),
        "1"
    );
    assert_eq!(
        format_number(val, &parse_number_format("mm").unwrap(), &default_loc()),
        "01"
    );
    assert_eq!(
        format_number(val, &parse_number_format("mmm").unwrap(), &default_loc()),
        "Jan"
    );
    assert_eq!(
        format_number(val, &parse_number_format("mmmm").unwrap(), &default_loc()),
        "January"
    );
    assert_eq!(
        format_number(val, &parse_number_format("mmmmm").unwrap(), &default_loc()),
        "J"
    ); // Fixed English letter
}

// --- Day/Weekday Representations ---
#[test]
fn test_weekday_formats() {
    let val = TEST_DATE_SERIAL; // 2024-01-01 is a Monday
    assert_eq!(
        format_number(val, &parse_number_format("d").unwrap(), &default_loc()),
        "1"
    ); // Day of month
    assert_eq!(
        format_number(val, &parse_number_format("dd").unwrap(), &default_loc()),
        "01"
    ); // Day of month
    assert_eq!(
        format_number(val, &parse_number_format("ddd").unwrap(), &default_loc()),
        "Mon"
    );
    assert_eq!(
        format_number(val, &parse_number_format("dddd").unwrap(), &default_loc()),
        "Monday"
    );
}

// --- Basic Time Tests (24-hour default) ---
#[test]
fn test_time_hh_mm_ss_24hr() {
    let fmt = parse_number_format("hh:mm:ss").unwrap();
    assert_eq!(
        format_number(TEST_DATE_SERIAL, &fmt, &default_loc()),
        "18:00:00"
    ); // 6 PM
}

#[test]
fn test_time_h_m_s_24hr() {
    let val = 45292.375; // 09:00:00
    let fmt = parse_number_format("h:m:s").unwrap();
    assert_eq!(format_number(val, &fmt, &default_loc()), "9:0:0");
}

// --- Time Tests with AM/PM (12-hour) ---
#[test]
fn test_time_hh_mm_am_pm_upper() {
    let fmt = parse_number_format("hh:mm AM/PM").unwrap();
    assert_eq!(
        format_number(TEST_DATE_SERIAL, &fmt, &default_loc()),
        "06:00 PM"
    ); // 6 PM
    assert_eq!(
        format_number(TEST_DATE_SERIAL_MORNING, &fmt, &default_loc()),
        "09:00 AM"
    );
}

#[test]
fn test_time_h_m_ss_am_pm_lower() {
    let fmt = parse_number_format("h:m:ss am/pm").unwrap();
    assert_eq!(
        format_number(TEST_DATE_SERIAL, &fmt, &default_loc()),
        "6:0:00 pm"
    );
    assert_eq!(
        format_number(TEST_DATE_SERIAL_MORNING, &fmt, &default_loc()),
        "9:0:00 am"
    );
}

#[test]
fn test_time_h_a_p_upper() {
    let fmt = parse_number_format("h A/P").unwrap();
    assert_eq!(format_number(TEST_DATE_SERIAL, &fmt, &default_loc()), "6 P");
    assert_eq!(
        format_number(TEST_DATE_SERIAL_MORNING, &fmt, &default_loc()),
        "9 A"
    );
}

#[test]
fn test_time_h_a_p_lower() {
    let fmt = parse_number_format("h a/p").unwrap();
    assert_eq!(format_number(TEST_DATE_SERIAL, &fmt, &default_loc()), "6 p");
    assert_eq!(
        format_number(TEST_DATE_SERIAL_MORNING, &fmt, &default_loc()),
        "9 a"
    );
}

// --- Duration Formats ---
#[test]
fn test_duration_h_mm_ss() {
    let fmt = parse_number_format("[h]:mm:ss").unwrap();
    assert_eq!(
        format_number(TEST_DURATION_SERIAL, &fmt, &default_loc()),
        "37:02:14"
    );
}

#[test]
fn test_duration_negative_input_error() {
    let fmt = parse_number_format("[h]:mm").unwrap();
    assert_eq!(
        format_number(-1.0, &fmt, &default_loc()),
        "ERROR: Negative value (-1) not allowed for duration format."
    );
}

// --- Literal and Quoted Text ---
#[test]
fn test_date_with_literals() {
    let fmt = parse_number_format("yyyy/mm/dd \"at\" hh:mm AM/PM").unwrap();
    assert_eq!(
        format_number(TEST_DATE_SERIAL, &fmt, &default_loc()),
        "2024/01/01 at 06:00 PM"
    );
}

// --- Localization Test ---
#[test]
fn test_french_localization_date() {
    let loc = french_loc();
    let fmt = parse_number_format("dddd d mmmm yyyy, hh:mm am/pm").unwrap();
    println!("{:?}", fmt.positive_section.tokens);
    assert_eq!(
        format_number(TEST_DATE_SERIAL, &fmt, &loc),
        "Lundi 1 janvier 2024, 06:00 apr.m."
    );
}

// --- Edge Cases ---
#[test]
fn test_time_around_midnight_pm() {
    let fmt = parse_number_format("yyyy-mm-dd h:mm:ss AM/PM").unwrap();
    assert_eq!(
        format_number(TEST_DATE_NEAR_MIDNIGHT, &fmt, &default_loc()),
        "2024-01-01 11:59:59 PM"
    );
}

#[test]
fn test_time_around_noon() {
    let noon = 45292.5; // 2024-01-01 12:00:00 PM
    let fmt = parse_number_format("h AM/PM").unwrap();
    assert_eq!(format_number(noon, &fmt, &default_loc()), "12 PM");

    let pre_noon = 45292.49999; // 2024-01-01 11:59:59 AM (approx)
    assert_eq!(format_number(pre_noon, &fmt, &default_loc()), "11 AM");
}

#[test]
fn test_excel_1900_bug_date() {
    let fmt_d = parse_number_format("yyyy-mm-dd").unwrap();
    // Excel considers serial 60 to be 1900-02-29 (its phantom leap day)
    assert_eq!(format_number(60.0, &fmt_d, &default_loc()), "1900-02-29");
    // Serial 59 is 1900-02-28
    assert_eq!(format_number(59.0, &fmt_d, &default_loc()), "1900-02-28");
    // Serial 61 is 1900-03-01
    assert_eq!(format_number(61.0, &fmt_d, &default_loc()), "1900-03-01");
}
