//! Texture description files: the sprites an XML description declares, and the two directions over them.

mod description_sprite_options;
mod description_sprite_packer;
mod description_sprite_result;
mod description_sprite_unpacker;
mod description_xml;
mod texture_file_descriptor;
mod texture_sprite_descriptor;
mod xml_description_collection;

pub use description_sprite_options::DescriptionSpriteOptions;
pub use description_sprite_packer::DescriptionSpritePacker;
pub use description_sprite_result::DescriptionSpriteResult;
pub use description_sprite_unpacker::DescriptionSpriteUnpacker;
pub(crate) use description_xml::*;
pub use texture_file_descriptor::TextureFileDescriptor;
pub use texture_sprite_descriptor::TextureSpriteDescriptor;
pub(crate) use xml_description_collection::XmlDescriptionCollection;
