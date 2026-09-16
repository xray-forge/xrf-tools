use serde::{Deserialize, Serialize};
use xrf_error::XrfResult;
use xrf_utils::encode_w1251_bytes_to_string;

/// Flags a compiler shader carries, by the bit `Shader_xrLC` names it under.
pub const SHADER_COMPILER_FLAGS: [(u32, &str); 6] = [
  (1 << 0, "collision"),
  (1 << 1, "rendering"),
  (1 << 2, "optimize UV"),
  (1 << 3, "vertex lighting"),
  (1 << 4, "casts shadow"),
  (1 << 5, "sharp light"),
];

/// One compiler shader, `Shader_xrLC` (`utils/Shader_xrLC.h`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShaderCompilerShader {
  pub name: String,
  pub flags: u32,
  pub vertex_translucency: f32,
  pub vertex_ambient: f32,
  /// Lightmap texels per unit, which is what a surface costs to bake.
  pub lightmap_density: f32,
}

impl ShaderCompilerShader {
  /// Bytes the fixed name field occupies, which is `char Name[128]` whatever the name's own length.
  pub const NAME_SIZE: usize = 128;

  /// Bytes one record occupies, which is what makes the file a plain array.
  pub const SERIALIZED_SIZE: u64 = Self::NAME_SIZE as u64 + 4 * 4;

  /// Decodes a name out of the fixed field holding it.
  ///
  /// The field is `char Name[128]` whatever the name's length, so the bytes past its terminator are padding and are
  /// not part of the name. Nothing else in these formats stores a string this way: every other one is terminated and
  /// occupies only what it needs.
  ///
  /// # Errors
  ///
  /// Returns an error when the bytes are not a Windows-1251 string.
  pub fn read_name(bytes: &[u8]) -> XrfResult<String> {
    let end: usize = bytes.iter().position(|byte| *byte == 0).unwrap_or(bytes.len());

    Ok(encode_w1251_bytes_to_string(&bytes[..end])?)
  }

  /// The flags the shader sets, named.
  pub fn get_named_flags(&self) -> Vec<&'static str> {
    SHADER_COMPILER_FLAGS
      .into_iter()
      .filter_map(|(mask, name)| (self.flags & mask != 0).then_some(name))
      .collect()
  }
}
