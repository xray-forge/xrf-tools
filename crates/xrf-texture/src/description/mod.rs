//! Texture description files: the sprites an XML description declares, and the two directions over them.

mod description_xml;
mod pack_description_options;
mod pack_description_processor;
mod texture_file_descriptor;
mod texture_sprite_descriptor;
mod unpack_description_processor;
mod xml_description_collection;

pub use pack_description_options::PackDescriptionOptions;
pub use pack_description_processor::PackDescriptionProcessor;
pub use texture_file_descriptor::TextureFileDescriptor;
pub use texture_sprite_descriptor::TextureSpriteDescriptor;
pub use unpack_description_processor::UnpackDescriptionProcessor;

pub(crate) use description_xml::*;
pub(crate) use xml_description_collection::XmlDescriptionCollection;
