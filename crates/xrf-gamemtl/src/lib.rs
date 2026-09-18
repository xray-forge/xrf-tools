//! `gamemtl.xr` game materials and the pairs that decide what happens when two meet.

pub(crate) mod gamemtl_acoustics;
pub(crate) mod gamemtl_file;
pub(crate) mod gamemtl_material;
pub(crate) mod gamemtl_pair;
#[cfg(test)]
mod tests;

pub use crate::gamemtl_acoustics::*;
pub use crate::gamemtl_file::*;
pub use crate::gamemtl_material::*;
pub use crate::gamemtl_pair::*;
