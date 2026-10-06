//! What shaders and Rust share, declared once in Rust: types laid out by WGSL's rules, their WGSL declarations, and a
//! check of both against naga.

mod generated_shader_file;
mod shader_address_space;
mod shader_declarations;
mod shader_layout;
mod shader_layout_verifier;
mod shader_member;
mod shader_struct;
mod shader_type;
mod shader_type_builtins;

#[cfg(test)]
mod tests;

pub use generated_shader_file::GeneratedShaderFile;
pub use shader_address_space::ShaderAddressSpace;
pub use shader_declarations::ShaderDeclarations;
pub use shader_layout::{max_of, round_up};
pub use shader_layout_verifier::ShaderLayoutVerifier;
pub use shader_member::ShaderMember;
pub use shader_struct::ShaderStruct;
pub use shader_type::ShaderType;
