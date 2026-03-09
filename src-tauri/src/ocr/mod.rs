pub mod capture;
pub mod parser;
pub mod recognize;

pub use capture::capture_screen_region;
pub use parser::parse_stats_from_text;
pub use recognize::recognize_text_from_image;
