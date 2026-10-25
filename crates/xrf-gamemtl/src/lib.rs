#![doc = include_str!("../README.md")]

pub(crate) mod gamemtl_acoustics;
pub(crate) mod gamemtl_file;
pub(crate) mod gamemtl_material;
pub(crate) mod gamemtl_pair;
pub(crate) mod gamemtl_version;
#[cfg(test)]
mod tests;

pub use crate::gamemtl_acoustics::*;
pub use crate::gamemtl_file::*;
pub use crate::gamemtl_material::*;
pub use crate::gamemtl_pair::*;
pub use crate::gamemtl_version::*;
