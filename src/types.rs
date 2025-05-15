//! Type definitions for number format parser
//!
//! This module defines the type system used to represent the parsed results of format strings.
//! Includes tokens, comparison operators, and format sections.

/// Represents a single format token parsed from the format string
#[derive(Debug, Clone, PartialEq)]
pub enum FormatToken {
    /// Number placeholder (0) that shows zero if no digit exists
    DigitOrZero,
    /// Number placeholder (#) that shows nothing if no digit exists
    DigitIfNeeded,
    /// Number placeholder (?) that shows a space if no digit exists
    DigitOrSpace,
    /// Decimal point (.)
    DecimalPoint,
    /// Thousands separator (,)
    ThousandsSeparator,
    /// Percentage symbol (%)
    Percentage,
    /// Exponential notation, such as E+ or E-
    Exponential(ExponentialNotation),
    /// Literal character to display directly
    LiteralChar(char),
    /// Fill character, * followed by a character
    Fill(char),
    /// Skip width, _ followed by a character
    SkipWidth(char),
    /// Quoted text, like "text"
    QuotedText(String),
    /// Text value placeholder (@)
    TextValue,

    /// Color indicator, like [Red], [Blue], etc.
    Color(ColorType),

    /// Two-digit year (yy)
    YearTwoDigit,
    /// Four-digit year (yyyy)
    YearFourDigit,
    /// Month number, e.g., January is 1 (m)
    MonthNum,
    /// Zero-padded month number, e.g., January is 01 (mm)
    MonthNumPadded,
    /// Month abbreviation, e.g., Jan (mmm)
    MonthAbbr,
    /// Full month name, e.g., January (mmmm)
    MonthFullName,
    /// Month initial letter, e.g., J (mmmmm)
    MonthLetter,
    /// Day number, 1-31 (d)
    DayNum,
    /// Zero-padded day number, 01-31 (dd)
    DayNumPadded,
    /// Weekday abbreviation, e.g., Mon (ddd)
    WeekdayAbbr,
    /// Full weekday name, e.g., Monday (dddd)
    WeekdayFullName,
    /// Hour in 12 or 24-hour format (h)
    Hour12Or24,
    /// Zero-padded hour in 12 or 24-hour format (hh)
    Hour12Or24Padded,
    /// Minute number (m), determined to be minutes from context
    MinuteNum,
    /// Zero-padded minute number (mm), determined to be minutes from context
    MinuteNumPadded,
    /// Second number (s)
    SecondNum,
    /// Zero-padded second number (ss)
    SecondNumPadded,
    /// AM/PM indicator, with style (uppercase/lowercase)
    AmPm(AmPmStyle),
    /// A/P indicator, with style (uppercase/lowercase)
    AP(AmPmStyle),
    /// Elapsed hours [h]
    ElapsedHours,
    /// Elapsed minutes [m]
    ElapsedMinutes,
    /// Elapsed seconds [s]
    ElapsedSeconds,

    /// Single m, might be month or minute, to be determined by context
    MonthOrMinute1,
    /// Double m, might be month or minute, to be determined by context
    MonthOrMinute2,
}

/// Represents the style (case) for AM/PM or A/P markers
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AmPmStyle {
    /// Uppercase style (AM, PM, A, P)
    UpperCase,
    /// Lowercase style (am, pm, a, p)
    LowerCase,
}

/// Represents color types
#[derive(Debug, Clone, PartialEq)]
pub enum ColorType {
    /// Red color
    Red,
    /// Green color
    Green,
    /// Blue color
    Blue,
    /// Magenta color
    Magenta,
    /// Cyan color
    Cyan,
    /// Yellow color
    Yellow,
    /// Black color
    Black,
    /// White color
    White,
}

impl FormatToken {
    /// Checks if the token is related to numbers or dates
    pub fn is_numeric_or_date(&self) -> bool {
        matches!(
            self,
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
                | FormatToken::ElapsedHours
                | FormatToken::ElapsedMinutes
                | FormatToken::ElapsedSeconds
                | FormatToken::MonthOrMinute1
                | FormatToken::MonthOrMinute2
                | FormatToken::DigitOrZero
                | FormatToken::DigitIfNeeded
                | FormatToken::DigitOrSpace
                | FormatToken::DecimalPoint
        )
    }

    /// Checks if the token is a placeholder for a date or time
    pub fn is_datetime_placeholder(&self) -> bool {
        matches!(
            self,
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
                | FormatToken::ElapsedHours
                | FormatToken::ElapsedMinutes
                | FormatToken::ElapsedSeconds
                | FormatToken::MonthOrMinute1
                | FormatToken::MonthOrMinute2
        )
    }
}

/// Type of exponential notation
#[derive(Debug, Clone, PartialEq)]
pub enum ExponentialNotation {
    /// E+ notation
    Plus,
    /// E- notation
    Minus,
}

/// Comparison operators for conditional formatting
#[derive(Debug, Clone, PartialEq)]
pub enum ComparisonOperator {
    /// Equal to (=)
    Eq,
    /// Greater than (>)
    Gt,
    /// Less than (<)
    Lt,
    /// Greater than or equal to (>=)
    Ge,
    /// Less than or equal to (<=)
    Le,
    /// Not equal to (<>)
    Ne,
}

/// Represents a format condition with an operator and a comparison value
#[derive(Debug, Clone, PartialEq)]
pub struct Condition {
    /// The comparison operator
    pub operator: ComparisonOperator,
    /// The value to compare against
    pub value: f64,
}

/// Represents a section of the format string with optional condition and a sequence of tokens
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FormatSection {
    /// The section's color, if any
    pub color: Option<ColorType>,
    /// The section's condition, if any
    pub condition: Option<Condition>,
    /// Sequence of format tokens
    pub tokens: Vec<FormatToken>,
    /// Indicates if this is the fourth section (text section)
    pub is_text_section: bool,
    /// Number of commas at the end of number placeholders for scaling
    pub num_scaling_commas: u8,
    pub has_datetime: bool,    // True if section contains any date/time tokens
    pub has_text_format: bool, // True if section contains @ (text format)
    pub has_fraction: bool,    // True if section contains / (for fractions, not date)
    pub fixed_denominator: Option<u32>, // For fraction formats like #/16
    pub num_integer_part_tokens: usize, // Count of 0#? before decimal or for non-decimal numbers
    pub num_fractional_part_tokens: usize, // Count of 0#? after decimal
}

/// Represents a complete number format with all sections
#[derive(Debug, Clone, PartialEq)]
pub struct NumberFormat {
    /// Positive section format (required)
    pub positive_section: FormatSection,
    /// Negative section format (optional)
    pub negative_section: Option<FormatSection>,
    /// Zero value section format (optional)
    pub zero_section: Option<FormatSection>,
    /// Text section format (optional)
    pub text_section: Option<FormatSection>,
}

/// Locale-specific settings for number formatting.
#[derive(Debug, Clone, PartialEq)]
pub struct LocaleSettings {
    /// Character to use for the decimal point.
    pub decimal_point: char,
    /// Character to use for the thousands separator.
    pub thousands_separator: char,

    /// AM and PM markers, e.g., `["AM", "PM"]` or `["上午", "下午"]`.
    /// Should contain two elements: [AM_equivalent, PM_equivalent].
    pub ampm_markers: [String; 2],

    /// Short day names, Sunday to Saturday, e.g., `["Sun", "Mon", ..., "Sat"]`.
    /// Should contain 7 elements, starting with Sunday.
    pub short_day_names: [String; 7],

    /// Full day names, Sunday to Saturday, e.g., `["Sunday", "Monday", ..., "Saturday"]`.
    /// Should contain 7 elements, starting with Sunday.
    pub day_names: [String; 7],

    /// Short month names, January to December, e.g., `["Jan", "Feb", ..., "Dec"]`.
    /// Should contain 12 elements, starting with January.
    pub short_month_names: [String; 12],

    /// Full month names, January to December, e.g., `["January", "February", ..., "December"]`.
    /// Should contain 12 elements, starting with January.
    pub month_names: [String; 12],
}

impl Default for LocaleSettings {
    fn default() -> Self {
        LocaleSettings {
            decimal_point: '.',
            thousands_separator: ',',

            ampm_markers: ["AM".to_string(), "PM".to_string()],
            short_day_names: [
                "Sun".to_string(),
                "Mon".to_string(),
                "Tue".to_string(),
                "Wed".to_string(),
                "Thu".to_string(),
                "Fri".to_string(),
                "Sat".to_string(),
            ],
            day_names: [
                "Sunday".to_string(),
                "Monday".to_string(),
                "Tuesday".to_string(),
                "Wednesday".to_string(),
                "Thursday".to_string(),
                "Friday".to_string(),
                "Saturday".to_string(),
            ],
            short_month_names: [
                "Jan".to_string(),
                "Feb".to_string(),
                "Mar".to_string(),
                "Apr".to_string(),
                "May".to_string(),
                "Jun".to_string(),
                "Jul".to_string(),
                "Aug".to_string(),
                "Sep".to_string(),
                "Oct".to_string(),
                "Nov".to_string(),
                "Dec".to_string(),
            ],
            month_names: [
                "January".to_string(),
                "February".to_string(),
                "March".to_string(),
                "April".to_string(),
                "May".to_string(),
                "June".to_string(),
                "July".to_string(),
                "August".to_string(),
                "September".to_string(),
                "October".to_string(),
                "November".to_string(),
                "December".to_string(),
            ],
        }
    }
}

impl LocaleSettings {
    /// Sets the decimal point character.
    pub fn with_decimal_point(mut self, ch: char) -> Self {
        self.decimal_point = ch;
        self
    }

    /// Sets the thousands separator character.
    pub fn with_thousands_separator(mut self, ch: char) -> Self {
        self.thousands_separator = ch;
        self
    }

    /// Sets the AM/PM markers.
    /// Expects an array of two string slices: `[am_marker, pm_marker]`.
    pub fn with_ampm_markers(mut self, markers: [&str; 2]) -> Self {
        self.ampm_markers = [markers[0].to_string(), markers[1].to_string()];
        self
    }

    /// Sets the short day names (Sunday to Saturday).
    /// Expects an array of seven string slices.
    pub fn with_short_day_names(mut self, names: [&str; 7]) -> Self {
        self.short_day_names = names.map(|s| s.to_string());
        self
    }

    /// Sets the full day names (Sunday to Saturday).
    /// Expects an array of seven string slices.
    pub fn with_day_names(mut self, names: [&str; 7]) -> Self {
        self.day_names = names.map(|s| s.to_string());
        self
    }

    /// Sets the short month names (January to December).
    /// Expects an array of twelve string slices.
    pub fn with_short_month_names(mut self, names: [&str; 12]) -> Self {
        self.short_month_names = names.map(|s| s.to_string());
        self
    }

    /// Sets the full month names (January to December).
    /// Expects an array of twelve string slices.
    pub fn with_month_names(mut self, names: [&str; 12]) -> Self {
        self.month_names = names.map(|s| s.to_string());
        self
    }
}
