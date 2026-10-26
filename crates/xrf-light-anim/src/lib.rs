#![doc = include_str!("../README.md")]

mod light_anim_file;
mod light_anim_item;
mod light_anim_key;

#[cfg(test)]
mod tests;

pub use crate::light_anim_file::LightAnimFile;
pub use crate::light_anim_item::LightAnimItem;
pub use crate::light_anim_key::LightAnimKey;
