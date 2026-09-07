use crate::shader_library::shader_blender_property_kind::ShaderBlenderPropertyKind;
use crate::shader_library::shader_blender_token::ShaderBlenderToken;

/// The payload of one property, by the type that wrote it.
///
/// The five name-carrying types stay apart rather than folding into one string variant, because the type is what a
/// writer has to put back and what says whether a name is a texture, a transform or a constant.
#[derive(Clone, Debug, PartialEq)]
pub enum ShaderBlenderPropertyValue {
  /// A heading in the editor's property grid, carrying nothing.
  Marker,
  Matrix(String),
  Constant(String),
  Texture(String),
  Object(String),
  Text(String),
  /// An authored value and the range the editor clamps it to, `xrP_Integer` (`xrEngine/Properties.h`).
  Integer {
    value: i32,
    minimum: i32,
    maximum: i32,
  },
  Float {
    value: f32,
    minimum: f32,
    maximum: f32,
  },
  /// `xrP_BOOL`, stored as a four byte `BOOL`, so anything non-zero is true.
  Bool(bool),
  Token {
    selected: u32,
    items: Vec<ShaderBlenderToken>,
  },
  ClassId {
    selected: u64,
    items: Vec<u64>,
  },
}

impl ShaderBlenderPropertyValue {
  /// The type that wrote this payload, which is what a writer puts back in front of it.
  pub const fn kind(&self) -> ShaderBlenderPropertyKind {
    match self {
      Self::Marker => ShaderBlenderPropertyKind::Marker,
      Self::Matrix(_) => ShaderBlenderPropertyKind::Matrix,
      Self::Constant(_) => ShaderBlenderPropertyKind::Constant,
      Self::Texture(_) => ShaderBlenderPropertyKind::Texture,
      Self::Object(_) => ShaderBlenderPropertyKind::Object,
      Self::Text(_) => ShaderBlenderPropertyKind::Text,
      Self::Integer { .. } => ShaderBlenderPropertyKind::Integer,
      Self::Float { .. } => ShaderBlenderPropertyKind::Float,
      Self::Bool(_) => ShaderBlenderPropertyKind::Bool,
      Self::Token { .. } => ShaderBlenderPropertyKind::Token,
      Self::ClassId { .. } => ShaderBlenderPropertyKind::ClassId,
    }
  }
}
