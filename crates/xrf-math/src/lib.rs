#![doc = include_str!("../README.md")]

pub(crate) mod display_colour;
pub(crate) mod math_constants;
pub(crate) mod matrix_4x4;
pub(crate) mod vector_3d;

pub use crate::display_colour::{LUMINANCE_WEIGHTS, SRGB_DISPLAY_POWER, luminance, srgb_decoded, srgb_encoded};
pub use crate::math_constants::{EPS, EPS_L, EPS_S};
pub use crate::matrix_4x4::Matrix4x4;
pub use crate::vector_3d::Vector3d;
