//! An image on its way to or from a texture file: which format a path names, how it is fitted, and how it is written.

mod image_extensions;
mod image_fit;
mod image_writer;

pub(crate) use image_extensions::{DDS_EXTENSION, PNG_EXTENSION};
pub(crate) use image_fit::fit_image_into_bounds;
pub(crate) use image_writer::warn_on_reshaped_ui_dds;
pub(crate) use image_writer::{UI_MIPMAP_LEVELS, UI_MIPMAPS, save_image_as_ui_dds, save_image_as_ui_png};
