//! `.xr` light animation libraries: colour keys over time.

pub(crate) mod light_anim_file;
pub(crate) mod light_anim_item;
pub(crate) mod light_anim_key;
#[cfg(test)]
mod tests;

pub use crate::light_anim_file::*;
pub use crate::light_anim_item::*;
pub use crate::light_anim_key::*;
