//! The level compiler's shader table, `shaders_xrlc.xr`: a plain array of per-shader bake settings.

mod shader_compiler_file;
mod shader_compiler_shader;

pub use shader_compiler_file::ShaderCompilerFile;
pub use shader_compiler_shader::ShaderCompilerShader;
