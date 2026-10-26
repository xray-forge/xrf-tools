//! What a texture's `.thm` descriptor declares to the renderer: its bump pair, its detail association and its lighting
//! model, read the way `CTextureDescrMngr` and `CRender::texture_load` read them.

mod bump;
mod xray_detail_usage;
mod xray_material_declaration;
mod xray_material_descriptor;
mod xray_material_detail;
mod xray_material_resolver;

pub use bump::{
  XrayBumpFallback, XrayBumpMode, XrayBumpNaming, XrayBumpOutcome, XrayMaterialBump, XrayMaterialBumpInput,
};
pub use xray_detail_usage::XrayDetailUsage;
pub use xray_material_declaration::XrayMaterialDeclaration;
pub use xray_material_descriptor::XrayMaterialDescriptor;
pub use xray_material_detail::XrayMaterialDetail;
pub use xray_material_resolver::XrayMaterialResolver;
