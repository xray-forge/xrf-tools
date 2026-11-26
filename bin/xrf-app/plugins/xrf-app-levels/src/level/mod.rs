//! The levels open in the application, how a page asks for one, and how a query of one is answered.

mod level_answer;
mod level_open_request;
mod selected_levels;

pub(crate) use level_answer::LevelAnswer;
pub(crate) use level_open_request::LevelOpenRequest;
pub use selected_levels::SelectedLevels;
