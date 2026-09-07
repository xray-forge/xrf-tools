use std::fmt::{Display, Formatter, Result as FormatResult};

/// The eight character class tag of a blender, `CLASS_ID` as the engine packs it (`xrCore/clsid.h`).
///
/// `make_clsid` puts the first character in the most significant byte, so the value is one integer to compare and a
/// readable tag to print. The constants below mirror `Blender_CLSID.h` whole, including the classes a renderer maps to
/// nothing: a library may hold any of them, and the engine answers an unmapped one with
/// `! Renderer doesn't support blender` and draws nothing rather than failing to load
/// (`Layers/xrRender/ResourceManager_Loader.cpp:118`).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ShaderBlenderClass(u64);

impl ShaderBlenderClass {
  /// Characters one tag occupies, which is also the field's size on disk.
  pub const TAG_SIZE: usize = 8;

  // Level surfaces.
  pub const DEFAULT: Self = Self::of(b"LM      ");
  pub const DEFAULT_AREF: Self = Self::of(b"LM_AREF ");
  pub const VERT: Self = Self::of(b"V       ");
  pub const VERT_AREF: Self = Self::of(b"V_AREF  ");
  pub const LM_BMM_D: Self = Self::of(b"LmBmmD  ");
  pub const LA_EM_B: Self = Self::of(b"LaEmB   ");
  pub const LM_EB_B: Self = Self::of(b"LmEbB   ");
  pub const BMM_D: Self = Self::of(b"BmmD    ");
  pub const BMM_D_OLD: Self = Self::of(b"BmmDold ");

  // Model surfaces.
  pub const MODEL: Self = Self::of(b"MODEL   ");
  pub const MODEL_EB_B: Self = Self::of(b"MODELEbB");

  // Scenery grown or scattered by the renderer rather than placed as a visual.
  pub const DETAIL: Self = Self::of(b"D_STILL ");
  pub const TREE: Self = Self::of(b"D_TREE  ");

  // Effects and passes that are not surfaces of a mesh.
  pub const PARTICLE: Self = Self::of(b"PARTICLE");
  pub const SCREEN_SET: Self = Self::of(b"S_SET   ");
  pub const SCREEN_GRAY: Self = Self::of(b"S_GRAY  ");
  pub const LIGHT: Self = Self::of(b"LIGHT   ");
  pub const BLUR: Self = Self::of(b"BLUR    ");
  pub const SHADOW_TEXTURE: Self = Self::of(b"SH_TEX  ");
  pub const SHADOW_WORLD: Self = Self::of(b"SH_WORLD");

  // Editor overlays, which ship in the library the game reads.
  pub const EDITOR_WIRE: Self = Self::of(b"E_WIRE  ");
  pub const EDITOR_SELECTION: Self = Self::of(b"E_SEL   ");

  /// Packs eight characters the way `make_clsid` does.
  pub const fn of(tag: &[u8; Self::TAG_SIZE]) -> Self {
    let mut raw: u64 = 0;
    let mut index: usize = 0;

    while index < Self::TAG_SIZE {
      raw = (raw << 8) | tag[index] as u64;
      index += 1;
    }

    Self(raw)
  }

  pub const fn from_raw(raw: u64) -> Self {
    Self(raw)
  }

  pub const fn raw(self) -> u64 {
    self.0
  }

  /// The tag as it was authored, without the padding that fills it out to eight characters.
  ///
  /// Trailing spaces and nulls are dropped because they are how a shorter name is stored rather than part of it: the
  /// engine spells the class `"MODEL   "` and everything that shows it to a person shows `MODEL`. A byte no printable
  /// ASCII covers is replaced, so a corrupt tag prints rather than costing every caller a result type for a label.
  pub fn tag(self) -> String {
    let bytes: [u8; Self::TAG_SIZE] = self.0.to_be_bytes();
    let padding: usize = bytes
      .iter()
      .rposition(|byte| *byte != b' ' && *byte != 0)
      .map_or(0, |last| last + 1);

    bytes[..padding]
      .iter()
      .map(|byte| {
        if byte.is_ascii_graphic() || *byte == b' ' {
          *byte as char
        } else {
          '?'
        }
      })
      .collect()
  }
}

impl Display for ShaderBlenderClass {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
    write!(formatter, "{}", self.tag())
  }
}

#[cfg(test)]
mod tests {
  use super::ShaderBlenderClass;

  #[test]
  fn packs_a_tag_the_way_the_engine_does() {
    // `make_clsid("MODEL   ")` with the first character in the most significant byte.
    assert_eq!(ShaderBlenderClass::MODEL.raw(), 0x4D4F44454C202020);
    assert_eq!(
      ShaderBlenderClass::from_raw(0x4D4F44454C202020),
      ShaderBlenderClass::MODEL
    );
  }

  #[test]
  fn prints_a_tag_without_its_padding() {
    assert_eq!(ShaderBlenderClass::MODEL.tag(), "MODEL");
    assert_eq!(ShaderBlenderClass::MODEL_EB_B.tag(), "MODELEbB");
    assert_eq!(ShaderBlenderClass::DEFAULT_AREF.tag(), "LM_AREF");
    assert_eq!(ShaderBlenderClass::TREE.to_string(), "D_TREE");
  }

  #[test]
  fn prints_a_tag_no_ascii_covers() {
    assert_eq!(ShaderBlenderClass::from_raw(u64::MAX).tag(), "????????");
  }
}
