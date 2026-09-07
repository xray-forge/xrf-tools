use xrf_error::{XrfError, XrfResult};

/// Which of the engine's property types a record carries, `xrProperties` (`xrEngine/Properties.h`).
///
/// The discriminants are the values on disk. Every property is written as `u32` type, a null terminated name, and a
/// payload the type alone fixes, which is what makes a property stream walkable without knowing the blender class that
/// wrote it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ShaderBlenderPropertyKind {
  Marker = 0,
  Matrix = 1,
  Constant = 2,
  Texture = 3,
  Integer = 4,
  Float = 5,
  Bool = 6,
  Token = 7,
  ClassId = 8,
  Object = 9,
  /// `xrPID_STRING`, named for what it holds because `String` beside `str` reads as the Rust type.
  Text = 10,
}

impl ShaderBlenderPropertyKind {
  /// Bytes a `string64` field occupies, which is the payload of every name-carrying type.
  ///
  /// The `xrPID_*` comments claim only the name reaches the stream, but `xrPWRITE_PROP` writes `sizeof(data)` for
  /// every type alike (`xrEngine/Properties.h`), and the fields it is given - `IBlender::oT_Name`, `oT_xform` - are
  /// `string64` (`Layers/xrRender/Blender.h`).
  pub const TEXT_SIZE: usize = 64;

  /// Bytes one `xrP_TOKEN::Item` occupies: its id and a `string64` label (`xrEngine/Properties.h`).
  pub const TOKEN_ITEM_SIZE: usize = size_of::<u32>() + Self::TEXT_SIZE;

  /// The type of a raw discriminant, or an error naming it.
  ///
  /// `xrPID_MARKER_TEMPLATE` (11) is refused rather than guessed at: it is declared and never written or read by the
  /// engine or the SDK, so its payload has no witness, and a wrong size here would desynchronise the whole stream
  /// instead of failing one field.
  ///
  /// # Errors
  ///
  /// When the discriminant is not one of the eleven types a blender can carry.
  pub fn of(raw: u32) -> XrfResult<Self> {
    match raw {
      0 => Ok(Self::Marker),
      1 => Ok(Self::Matrix),
      2 => Ok(Self::Constant),
      3 => Ok(Self::Texture),
      4 => Ok(Self::Integer),
      5 => Ok(Self::Float),
      6 => Ok(Self::Bool),
      7 => Ok(Self::Token),
      8 => Ok(Self::ClassId),
      9 => Ok(Self::Object),
      10 => Ok(Self::Text),
      _ => Err(XrfError::new_not_implemented_error(format!(
        "Shader blender property type {raw} is not supported"
      ))),
    }
  }

  pub const fn raw(self) -> u32 {
    self as u32
  }
}

#[cfg(test)]
mod tests {
  use crate::shader_library::shader_blender_property_kind::ShaderBlenderPropertyKind;

  #[test]
  fn refuses_a_property_type_the_engine_never_writes() {
    // `xrPID_MARKER_TEMPLATE` and everything above it: declared in the engine, written by nothing, so no payload size
    // can be claimed for them.
    assert_eq!(
      ShaderBlenderPropertyKind::of(11).unwrap_err().to_string(),
      "Not implemented error: Shader blender property type 11 is not supported"
    );
    assert!(ShaderBlenderPropertyKind::of(64).is_err());
  }

  #[test]
  fn round_trips_every_discriminant_it_accepts() {
    for raw in 0..=10 {
      assert_eq!(ShaderBlenderPropertyKind::of(raw).expect("type is known").raw(), raw);
    }
  }
}
