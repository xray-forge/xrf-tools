//! The `_bump` and `_bump#` pair a bumped surface binds, generated the way the SDK's own converter does.

mod bump_normal_map;
mod bump_vector;
mod generate_bump_options;
mod generate_bump_processor;
mod generate_bump_result;

pub use generate_bump_options::{GenerateBumpGloss, GenerateBumpOptions};
pub use generate_bump_processor::GenerateBumpProcessor;
pub use generate_bump_result::GenerateBumpResult;
