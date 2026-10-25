//! What a save, a build or a bump generation came to.

mod texture_build_omission_report;
mod texture_build_outcome;
mod texture_make_bump_outcome;
mod texture_save_outcome;

pub use texture_build_omission_report::TextureBuildOmissionReport;
pub use texture_build_outcome::TextureBuildOutcome;
pub use texture_make_bump_outcome::TextureMakeBumpOutcome;
pub use texture_save_outcome::TextureSaveOutcome;
