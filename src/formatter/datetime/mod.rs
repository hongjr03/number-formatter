//! DateTime formatting module
//!
//! This module handles formatting of date and time values according to Excel format patterns.

mod conversion;
mod duration;
mod point_in_time;
mod utils;

// Re-export the public interface
pub use conversion::convert_f64_to_datetime;
pub use duration::{format_duration, section_is_duration};
pub use point_in_time::{format_datetime, section_is_datetime_point_in_time};
pub use utils::special_dates;
