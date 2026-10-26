//! An image on its way to or from a texture file: how it is read, how it is fitted, and how it is written.

mod image_file_reader;
mod image_file_writer;
mod image_fit;
mod image_shape;

pub use image_file_reader::ImageFileReader;
pub(crate) use image_file_writer::ImageFileWriter;
pub(crate) use image_fit::ImageFit;
pub use image_shape::ImageShape;
