//! Writing, which only the editor does: the descriptor fields a person edits, where an edit lands with the stamp of
//! what is there, saving a node's two files or refusing to, and rebuilding a texture or its bump pair from images.

pub(crate) mod commands;
mod texture_build_omission_report;
mod texture_build_outcome;
mod texture_descriptor_form;
mod texture_descriptor_save;
mod texture_edit_targets;
mod texture_encoding_save;
mod texture_make_bump_outcome;
mod texture_save_outcome;
mod texture_save_target;
mod texture_saver;
mod texture_write;
mod textures_build_request;
mod textures_make_bump_request;
mod textures_save_request;

pub use texture_build_omission_report::TextureBuildOmissionReport;
pub use texture_build_outcome::TextureBuildOutcome;
pub use texture_descriptor_form::TextureDescriptorForm;
pub use texture_descriptor_save::TextureDescriptorSave;
pub use texture_edit_targets::TextureEditTargets;
pub use texture_encoding_save::TextureEncodingSave;
pub use texture_make_bump_outcome::TextureMakeBumpOutcome;
pub use texture_save_outcome::TextureSaveOutcome;
pub use texture_save_target::TextureSaveTarget;
pub use texture_saver::TextureSaver;
pub use texture_write::TextureWrite;
pub use textures_build_request::TexturesBuildRequest;
pub use textures_make_bump_request::TexturesMakeBumpRequest;
pub use textures_save_request::TexturesSaveRequest;
