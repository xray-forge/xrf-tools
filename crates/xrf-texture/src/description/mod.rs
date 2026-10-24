//! Texture description files: the sprites an XML description declares, and the two directions over them.

mod description_xml;
mod pack_description_processor;
mod sprite_description_options;
mod sprite_description_result;
mod texture_file_descriptor;
mod texture_sprite_descriptor;
mod unpack_description_processor;
mod xml_description_collection;

pub(crate) use description_xml::*;
pub use pack_description_processor::PackDescriptionProcessor;
pub use sprite_description_options::SpriteDescriptionOptions;
pub use sprite_description_result::SpriteDescriptionResult;
pub use texture_file_descriptor::TextureFileDescriptor;
pub use texture_sprite_descriptor::TextureSpriteDescriptor;
pub use unpack_description_processor::UnpackDescriptionProcessor;
pub(crate) use xml_description_collection::XmlDescriptionCollection;
