//! What the X-Ray renderer builds for a surface, answered the way the engine would.
//!
//! Two halves, resolved from the two things a surface declares. From a texture's `.thm`: the bump pair, the detail
//! association, and the outcome of resolving them, the way `CTextureDescrMngr` and `CRender::texture_load` read them.
//! From a shader name and `shaders.xr`: whether alpha is read at all and how, the way the blender's own `Compile`
//! decides it. Neither is derivable from the other, and neither is in the mesh.

pub(crate) mod data;
pub(crate) mod resolve;

#[cfg(any(test, feature = "fixtures"))]
pub mod fixtures;

#[cfg(test)]
mod tests;

pub use crate::data::xray_bump_fallback::XrayBumpFallback;
pub use crate::data::xray_bump_mode::XrayBumpMode;
pub use crate::data::xray_bump_naming::XrayBumpNaming;
pub use crate::data::xray_bump_outcome::XrayBumpOutcome;
pub use crate::data::xray_detail_usage::XrayDetailUsage;
pub use crate::data::xray_material_bump::XrayMaterialBump;
pub use crate::data::xray_material_bump_input::XrayMaterialBumpInput;
pub use crate::data::xray_material_declaration::XrayMaterialDeclaration;
pub use crate::data::xray_material_descriptor::XrayMaterialDescriptor;
pub use crate::data::xray_material_detail::XrayMaterialDetail;
pub use crate::data::xray_surface_declaration::XraySurfaceDeclaration;
pub use crate::data::xray_surface_descriptor::XraySurfaceDescriptor;
pub use crate::data::xray_surface_draw::XraySurfaceDraw;
pub use crate::resolve::xray_material_resolver::XrayMaterialResolver;
pub use crate::resolve::xray_surface_resolver::XraySurfaceResolver;
