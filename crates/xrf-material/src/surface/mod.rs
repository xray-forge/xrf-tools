//! How the renderer draws a surface, from the shader name it declares: its renderer script, or else its blender in
//! `shaders.xr` and the rules that blender's class compiles by.

mod binding;
mod rule;
mod xray_surface_declaration;
mod xray_surface_descriptor;
mod xray_surface_draw;
mod xray_surface_resolver;
mod xray_surface_script;

pub use binding::{XraySurfaceDetail, XraySurfaceSampler, XraySurfaceTerrain, XraySurfaceTerrainLayer};
pub use xray_surface_declaration::XraySurfaceDeclaration;
pub use xray_surface_descriptor::XraySurfaceDescriptor;
pub use xray_surface_draw::XraySurfaceDraw;
pub use xray_surface_resolver::XraySurfaceResolver;
pub(crate) use xray_surface_script::XraySurfaceScript;
