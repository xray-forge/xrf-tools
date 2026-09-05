use serde::{Deserialize, Serialize};

/// Pixel format a descriptor asks the converter to build, `STextureParams::ETFormat` (`ETextureParams.h`).
///
/// Authoring data: the runtime reads the format out of the DDS header it loads, never out of the descriptor. A
/// descriptor whose format disagrees with the file beside it is a texture somebody re-encoded without rebuilding,
/// which is a thing a surface can say and not a thing to correct silently.
///
/// [`Self::NAMED`] is every member of the enum; [`Self::OFFERED`] is the shorter list the SDK's own format combo
/// offers (`tfmt_token`, `ETextureParams.cpp`). The four members outside it - [`Self::Rgba4444`], [`Self::Rgb`] and
/// the two `NVH` height formats - are reachable in a file but not through the editor that wrote it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "u32", into = "u32")]
pub enum ThmFormat {
  Dxt1,
  Dxt1Alpha,
  Dxt3,
  Dxt5,
  Rgba4444,
  Rgba1555,
  Rgb565,
  Rgb,
  Rgba,
  Nvhs,
  Nvhu,
  A8,
  L8,
  A8L8,
  /// A value the SDK has no name for, kept as it was read so a descriptor written back says what it said.
  Unknown(u32),
}

impl ThmFormat {
  /// Every format the SDK names, in declaration order.
  pub const NAMED: [Self; 14] = [
    Self::Dxt1,
    Self::Dxt1Alpha,
    Self::Dxt3,
    Self::Dxt5,
    Self::Rgba4444,
    Self::Rgba1555,
    Self::Rgb565,
    Self::Rgb,
    Self::Rgba,
    Self::Nvhs,
    Self::Nvhu,
    Self::A8,
    Self::L8,
    Self::A8L8,
  ];

  /// The formats the SDK's own editor offers, in the order it lists them.
  pub const OFFERED: [Self; 10] = [
    Self::Dxt1,
    Self::Dxt1Alpha,
    Self::Dxt3,
    Self::Dxt5,
    Self::Rgba1555,
    Self::Rgb565,
    Self::Rgba,
    Self::A8,
    Self::L8,
    Self::A8L8,
  ];

  /// Whether the layout carries an alpha channel at all, `STextureParams::HasAlphaChannel` (`ETextureParams.h:124`).
  ///
  /// A property of the layout, unlike `flHasAlpha`, which is the author saying the alpha means something.
  pub const fn has_alpha_channel(self) -> bool {
    matches!(
      self,
      Self::Dxt1Alpha | Self::Dxt3 | Self::Dxt5 | Self::Rgba4444 | Self::Rgba1555 | Self::Rgba
    )
  }

  /// Editor token for the format, `tfmt_token` (`ETextureParams.cpp:38`), falling back to the enum identifier for a
  /// member that table omits and to the number for a value the SDK has no name for.
  pub fn label(self) -> String {
    match self {
      Self::Dxt1 => String::from("DXT1"),
      Self::Dxt1Alpha => String::from("DXT1 Alpha"),
      Self::Dxt3 => String::from("DXT3"),
      Self::Dxt5 => String::from("DXT5"),
      Self::Rgba4444 => String::from("16 bit (4:4:4:4)"),
      Self::Rgba1555 => String::from("16 bit (1:5:5:5)"),
      Self::Rgb565 => String::from("16 bit (5:6:5)"),
      Self::Rgb => String::from("24 bit (8:8:8)"),
      Self::Rgba => String::from("32 bit (8:8:8:8)"),
      Self::Nvhs => String::from("NVHS"),
      Self::Nvhu => String::from("NVHU"),
      Self::A8 => String::from("8 bit (alpha)"),
      Self::L8 => String::from("8 bit (luminance)"),
      Self::A8L8 => String::from("16 bit (alpha:luminance)"),
      Self::Unknown(raw) => raw.to_string(),
    }
  }
}

impl Default for ThmFormat {
  /// `STextureParams::STextureParams` zeroes the struct, which is [`Self::Dxt1`] (`ETextureParams.h:114`).
  fn default() -> Self {
    Self::Dxt1
  }
}

impl From<u32> for ThmFormat {
  fn from(raw: u32) -> Self {
    match raw {
      0 => Self::Dxt1,
      1 => Self::Dxt1Alpha,
      2 => Self::Dxt3,
      3 => Self::Dxt5,
      4 => Self::Rgba4444,
      5 => Self::Rgba1555,
      6 => Self::Rgb565,
      7 => Self::Rgb,
      8 => Self::Rgba,
      9 => Self::Nvhs,
      10 => Self::Nvhu,
      11 => Self::A8,
      12 => Self::L8,
      13 => Self::A8L8,
      raw => Self::Unknown(raw),
    }
  }
}

impl From<ThmFormat> for u32 {
  fn from(format: ThmFormat) -> Self {
    match format {
      ThmFormat::Dxt1 => 0,
      ThmFormat::Dxt1Alpha => 1,
      ThmFormat::Dxt3 => 2,
      ThmFormat::Dxt5 => 3,
      ThmFormat::Rgba4444 => 4,
      ThmFormat::Rgba1555 => 5,
      ThmFormat::Rgb565 => 6,
      ThmFormat::Rgb => 7,
      ThmFormat::Rgba => 8,
      ThmFormat::Nvhs => 9,
      ThmFormat::Nvhu => 10,
      ThmFormat::A8 => 11,
      ThmFormat::L8 => 12,
      ThmFormat::A8L8 => 13,
      ThmFormat::Unknown(raw) => raw,
    }
  }
}
