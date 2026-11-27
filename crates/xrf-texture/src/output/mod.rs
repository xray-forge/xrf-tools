//! Where a processor puts each texture file it writes.

mod texture_disk_writer;
mod texture_file_writer;

pub use texture_disk_writer::TextureDiskWriter;
pub use texture_file_writer::TextureFileWriter;
