//! X-Ray shaders: authored sources, the compiled blender library, and the compiler's own table.

#[cfg(any(test, feature = "fixtures"))]
pub mod fixtures;
pub(crate) mod shader_blender;
pub(crate) mod shader_blender_class;
pub(crate) mod shader_blender_property;
pub(crate) mod shader_blender_property_kind;
pub(crate) mod shader_blender_property_value;
pub(crate) mod shader_blender_token;
pub(crate) mod shader_compiler_file;
pub(crate) mod shader_compiler_shader;
pub(crate) mod shader_fixed_string;
pub(crate) mod shader_library_file;

#[cfg(test)]
mod tests;

mod shader_renderer;
mod shader_source_config;
mod xray_shader;
mod xray_shader_compiler;
mod xray_shader_import;
mod xray_shader_import_reference;
mod xray_shader_pass;
mod xray_shader_script;
mod xray_shader_source_loader;

pub use shader_renderer::*;
pub use shader_source_config::*;
pub use xray_shader::*;
pub use xray_shader_compiler::*;
pub use xray_shader_import::*;
pub use xray_shader_pass::*;
pub use xray_shader_script::*;
pub use xray_shader_source_loader::*;

pub use crate::shader_blender::*;
pub use crate::shader_blender_class::*;
pub use crate::shader_blender_property::*;
pub use crate::shader_blender_property_kind::*;
pub use crate::shader_blender_property_value::*;
pub use crate::shader_blender_token::*;
pub use crate::shader_library_file::*;

pub use crate::shader_compiler_file::*;
pub use crate::shader_compiler_shader::*;
