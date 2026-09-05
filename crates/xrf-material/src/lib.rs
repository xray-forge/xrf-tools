//! What the X-Ray renderer builds for a texture from its descriptor: the bump pair, the detail association, and the
//! outcome of resolving them, answered the way `CTextureDescrMngr` and `CRender::texture_load` would.

pub(crate) mod data;
pub(crate) mod resolve;

#[cfg(any(test, feature = "fixtures"))]
pub mod fixtures;

#[cfg(test)]
mod tests;

pub use crate::data::xray_bump_fallback::XrayBumpFallback;
pub use crate::data::xray_bump_mode::XrayBumpMode;
pub use crate::data::xray_bump_outcome::XrayBumpOutcome;
pub use crate::data::xray_detail_usage::XrayDetailUsage;
pub use crate::data::xray_material_bump::XrayMaterialBump;
pub use crate::data::xray_material_bump_input::XrayMaterialBumpInput;
pub use crate::data::xray_material_declaration::XrayMaterialDeclaration;
pub use crate::data::xray_material_descriptor::XrayMaterialDescriptor;
pub use crate::data::xray_material_detail::XrayMaterialDetail;
pub use crate::resolve::xray_material_resolver::XrayMaterialResolver;
