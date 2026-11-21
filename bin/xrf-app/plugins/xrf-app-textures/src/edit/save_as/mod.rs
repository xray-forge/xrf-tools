//! Save as: planning a copy of a texture under a new name, and writing it whole or not at all.

mod texture_reference_field;
mod texture_reference_rename;
mod texture_save_as_bump;
mod texture_save_as_file;
mod texture_save_as_plan;
mod texture_save_as_role;
mod texture_save_as_writer;

pub(crate) use texture_reference_field::TextureReferenceField;
pub(crate) use texture_reference_rename::TextureReferenceRename;
pub(crate) use texture_save_as_bump::TextureSaveAsBump;
pub(crate) use texture_save_as_file::TextureSaveAsFile;
pub(crate) use texture_save_as_plan::TextureSaveAsPlan;
pub(crate) use texture_save_as_role::TextureSaveAsRole;
pub(crate) use texture_save_as_writer::TextureSaveAsWriter;
