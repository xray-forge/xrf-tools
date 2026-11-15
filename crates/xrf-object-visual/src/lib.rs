#![doc = include_str!("../README.md")]

pub(crate) mod object_visual;
#[cfg(test)]
mod tests;
pub(crate) mod visual_surfaces;

pub use crate::object_visual::ObjectVisual;
pub use crate::visual_surfaces::VisualSurfaces;
