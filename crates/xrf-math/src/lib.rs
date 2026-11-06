#![doc = include_str!("../README.md")]

pub(crate) mod affine_3d;
pub(crate) mod color;
pub(crate) mod display_colour;
pub(crate) mod math_constants;
pub(crate) mod matrix_4x4;
pub(crate) mod shape;
pub(crate) mod vector_2d;
pub(crate) mod vector_3d;

pub use crate::affine_3d::Affine3d;
pub use crate::color::Color;
pub use crate::display_colour::{LUMINANCE_WEIGHTS, SRGB_DISPLAY_POWER, luminance, srgb_decoded, srgb_encoded};
pub use crate::math_constants::{EPS, EPS_L, EPS_S};
pub use crate::matrix_4x4::Matrix4x4;
pub use crate::shape::aabb::Aabb;
pub use crate::shape::cylinder::Cylinder;
pub use crate::shape::obb::Obb;
pub use crate::shape::sphere::Sphere;
pub use crate::vector_2d::Vector2d;
pub use crate::vector_3d::Vector3d;
