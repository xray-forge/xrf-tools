pub(crate) mod data;
pub(crate) mod pack;
pub(crate) mod resolve;

pub use crate::data::visual_bounds::{VisualBounds, VisualBox, VisualSphere};
pub use crate::data::visual_description::{VisualBone, VisualDescription, VisualTransform};
pub use crate::data::visual_section::{VisualDrawRange, VisualSection};
pub use crate::data::visual_submesh::{
  VisualGeometry, VisualSkin, VisualSkipCause, VisualSubmesh, VisualSubmeshContent,
};
pub use crate::pack::visual_buffer_builder::VisualBufferBuilder;
pub use crate::pack::visual_conversion::{convert_declared_bounds, convert_texture_coordinates, convert_vector};
pub use crate::pack::visual_motion::{
  FLOATS_PER_BONE, VisualMotionBake, VisualMotionPose, bake_motion, total_part_bones,
};
pub use crate::pack::visual_package::VisualPackage;
pub use crate::pack::visual_packer::VisualPacker;
pub use crate::resolve::visual_dependencies::{VisualDependencies, VisualMotionDependency, VisualTextureDependency};
