//! HLSL and GLSL sources with their includes resolved as one renderer resolves them, and the interface a compiler
//! backend implements.

mod shader_renderer;
mod xray_shader;
mod xray_shader_compiler;
mod xray_shader_import;
mod xray_shader_import_reference;
mod xray_shader_placeholder_compiler;
mod xray_shader_source_loader;

pub use shader_renderer::ShaderRenderer;
pub use xray_shader::XRayShader;
pub use xray_shader_compiler::XRayShaderCompiler;
pub use xray_shader_import::XRayShaderImport;
pub(crate) use xray_shader_import_reference::XRayShaderImportReference;
pub use xray_shader_placeholder_compiler::XRayShaderPlaceholderCompiler;
pub use xray_shader_source_loader::XRayShaderSourceLoader;
