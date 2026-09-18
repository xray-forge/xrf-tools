pub(crate) mod data;
pub(crate) mod pack;
pub(crate) mod resolve;

pub use crate::data::sector_description::SectorDescription;
pub use crate::data::sector_geometry::SectorGeometry;
pub use crate::data::sector_instance_group::SectorInstanceGroup;
pub use crate::data::sector_outline::SectorOutline;
pub use crate::data::sector_section::SectorSection;
pub use crate::data::sector_skip::SectorSkip;
pub use crate::data::sector_surface::SectorSurface;
pub use crate::data::visual_bounds::{VisualBounds, VisualBox, VisualSphere};
pub use crate::data::visual_description::{VisualBone, VisualDescription, VisualTransform};
pub use crate::data::visual_section::{VisualDrawRange, VisualSection};
pub use crate::data::visual_submesh::{
  VisualGeometry, VisualSkin, VisualSkipCause, VisualSubmesh, VisualSubmeshContent,
};
pub use crate::pack::sector_package::SectorPackage;
pub use crate::pack::sector_packer::SectorPacker;
pub use crate::pack::visual_buffer_builder::VisualBufferBuilder;
pub use crate::pack::visual_conversion::{convert_declared_bounds, convert_uvs, convert_vector};
pub use crate::pack::visual_motion::{
  FLOATS_PER_BONE, VisualMotionBake, VisualMotionPose, bake_motion, total_part_bones,
};
pub use crate::pack::visual_package::VisualPackage;
pub use crate::pack::visual_packer::VisualPacker;
pub use crate::resolve::visual_dependencies::{VisualDependencies, VisualMotionDependency, VisualTextureDependency};
