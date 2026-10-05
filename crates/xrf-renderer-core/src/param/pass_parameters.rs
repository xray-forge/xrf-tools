use crate::graph::{GraphBuffer, GraphBufferAccess, GraphTexture, GraphTextureAccess};
use crate::param::pass_binding::PassBinding;
use crate::param::pass_resources::PassResources;
use crate::shader::ShaderDeclarations;

/// One bind group of a pass, declared once as a struct (`#[derive(PassParameters)]`): its layout, the WGSL that
/// declares its bindings, the graph accesses it makes, and the resources it binds, a binding per field in order.
pub trait PassParameters {
  /// The bind group index its WGSL declarations name, and the one it is bound at.
  const GROUP: u32;
  /// What the bind group cache knows its layout by: the struct's path.
  const LAYOUT_KEY: &'static str;

  fn get_layout_entries() -> Vec<wgpu::BindGroupLayoutEntry>;

  /// `@group(G) @binding(i) var …;` for each field, named as the field is.
  fn get_wgsl_bindings() -> String;

  /// Adds the structs its uniforms and storage arrays hold.
  fn declare(declarations: &mut ShaderDeclarations);

  fn list_texture_accesses(&self) -> Vec<(GraphTexture, GraphTextureAccess)>;

  fn list_buffer_accesses(&self) -> Vec<(GraphBuffer, GraphBufferAccess)>;

  /// What each binding binds this frame.
  fn list_bindings<'r>(&'r self, resources: &dyn PassResources<'r>) -> Vec<PassBinding<'r>>;

  /// The uniforms' offsets into their upload ring, in binding order.
  fn list_dynamic_offsets(&self) -> Vec<u32>;
}
