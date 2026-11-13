use serde::{Deserialize, Serialize};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::{decode_w1251_field_to_string, encode_string_to_w1251_bytes};

/// One compiler shader, `Shader_xrLC` (`utils/Shader_xrLC.h`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShaderCompilerShader {
  pub name: String,
  /// What the editor left in the name field past the terminator, kept so a library rewrites byte for byte.
  pub name_trailing: Vec<u8>,
  pub flags: u32,
  pub vertex_translucency: f32,
  pub vertex_ambient: f32,
  /// Lightmap texels per unit, which is what a surface costs to bake.
  pub lightmap_density: f32,
}

impl ShaderCompilerShader {
  /// Flags a compiler shader carries, by the bit `Shader_xrLC` names it under, with a name to show.
  pub const FLAGS: [(u32, &'static str); 6] = [
    (Self::COLLISION, "collision"),
    (Self::RENDERING, "rendering"),
    (Self::OPTIMIZE_UV, "optimize UV"),
    (Self::VERTEX_LIGHTING, "vertex lighting"),
    (Self::CASTS_SHADOW, "casts shadow"),
    (Self::SHARP_LIGHT, "sharp light"),
  ];

  /// `bCollision`: xrLC puts a face of this shader into the level's collision form, `level.cform`.
  pub const COLLISION: u32 = 1 << 0;

  /// `bRendering`: xrLC draws a face of this shader, putting it into the level's visuals.
  pub const RENDERING: u32 = 1 << 1;

  /// `bOptimizeUV`: xrLC moves a visual's base coordinates by whole tiles toward the origin, a connected group at a
  /// time.
  pub const OPTIMIZE_UV: u32 = 1 << 2;

  /// `bLIGHT_Vertex`: a face of this shader is lit by vertex colours, not a lightmap.
  pub const VERTEX_LIGHTING: u32 = 1 << 3;

  /// `bLIGHT_CastShadow`: the light compilers stop a ray at a face of this shader, or let it through when clear, and
  /// xrLC puts it into the ray model, `build.cform`.
  pub const CASTS_SHADOW: u32 = 1 << 4;

  /// `bLIGHT_Sharp`: the light baker lights a face of this shader without softening.
  pub const SHARP_LIGHT: u32 = 1 << 5;

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

    Ok((decode_w1251_field_to_string(&bytes[..end])?, bytes[end..].to_vec()))
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

  /// Whether a face of this shader collides, [`Self::COLLISION`].
  pub fn is_collidable(&self) -> bool {
    self.flags & Self::COLLISION != 0
  }

  /// Whether a face of this shader is drawn, [`Self::RENDERING`].
  pub fn is_rendered(&self) -> bool {
    self.flags & Self::RENDERING != 0
  }

  /// Whether a visual of this shader has its base coordinates moved toward the origin, [`Self::OPTIMIZE_UV`].
  pub fn optimizes_uv(&self) -> bool {
    self.flags & Self::OPTIMIZE_UV != 0
  }

  /// Whether a face of this shader is lit by vertex colours, [`Self::VERTEX_LIGHTING`].
  pub fn is_vertex_lit(&self) -> bool {
    self.flags & Self::VERTEX_LIGHTING != 0
  }

  /// Whether a face of this shader casts shadows, [`Self::CASTS_SHADOW`].
  pub fn casts_shadow(&self) -> bool {
    self.flags & Self::CASTS_SHADOW != 0
  }

  /// Whether a face of this shader is lit without softening, [`Self::SHARP_LIGHT`].
  pub fn is_sharply_lit(&self) -> bool {
    self.flags & Self::SHARP_LIGHT != 0
  }

  /// The flags the shader sets, named.
  pub fn get_named_flags(&self) -> Vec<&'static str> {
    Self::FLAGS
      .into_iter()
      .filter_map(|(mask, name)| (self.flags & mask != 0).then_some(name))
      .collect()
  }
}
