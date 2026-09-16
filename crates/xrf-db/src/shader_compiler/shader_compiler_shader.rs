use serde::{Deserialize, Serialize};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::{encode_string_to_w1251_bytes, encode_w1251_bytes_to_string};

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
  /// What the editor left in the name field past the terminator, kept so a library rewrites byte for byte.
  ///
  /// `Shader_xrLC` writes the whole `char Name[128]` out of a struct it never zeroes, so the bytes after a shorter
  /// name are whatever the field held before - vanilla's first record still carries a `0` from an earlier name.
  /// They are not part of the name, and dropping them would rewrite every shipped library differently.
  pub name_trailing: Vec<u8>,
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

  /// Splits a name field into the name and the bytes trailing it.
  ///
  /// # Errors
  ///
  /// Returns an error when the field carries no terminator, or is not a Windows-1251 string.
  pub fn read_name(bytes: &[u8]) -> XrfResult<(String, Vec<u8>)> {
    let Some(end) = bytes.iter().position(|byte| *byte == 0) else {
      return Err(XrfError::new_no_terminator_error(
        "Compiler shader name is not null terminated",
      ));
    };

    Ok((encode_w1251_bytes_to_string(&bytes[..end])?, bytes[end..].to_vec()))
  }

  /// Rebuilds the name field out of the name and what trailed it.
  ///
  /// # Errors
  ///
  /// Returns an error when the two do not fill the field exactly, which means one of them was changed without the
  /// other.
  pub fn write_name(&self) -> XrfResult<Vec<u8>> {
    let mut bytes: Vec<u8> = encode_string_to_w1251_bytes(&self.name)?;

    bytes.extend_from_slice(&self.name_trailing);

    if bytes.len() != Self::NAME_SIZE {
      return Err(XrfError::new_invalid_error(format!(
        "Compiler shader name '{}' and its {} trailing bytes do not fill the {}-byte field holding it",
        self.name,
        self.name_trailing.len(),
        Self::NAME_SIZE
      )));
    }

    Ok(bytes)
  }

  /// The flags the shader sets, named.
  pub fn get_named_flags(&self) -> Vec<&'static str> {
    SHADER_COMPILER_FLAGS
      .into_iter()
      .filter_map(|(mask, name)| (self.flags & mask != 0).then_some(name))
      .collect()
  }
}
