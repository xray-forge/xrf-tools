use serde::{Deserialize, Serialize};

/// Filter the converter reduces each mip level with, `kMIPFilter*` (`ETextureParams.h`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "u32", into = "u32")]
pub enum ThmMipFilter {
  Box,
  Cubic,
  Point,
  Triangle,
  Quadratic,
  Advanced,
  Catrom,
  Mitchell,
  Gaussian,
  Sinc,
  Bessel,
  Hanning,
  Hamming,
  Blackman,
  Kaiser,
  /// A value the SDK has no name for, kept as it was read so a descriptor written back says what it said.
  Unknown(u32),
}

impl ThmMipFilter {
  /// Every filter the SDK names, in the order its own combo lists them (`tparam_token`, `ETextureParams.cpp`).
  pub const NAMED: [Self; 15] = [
    Self::Advanced,
    Self::Point,
    Self::Box,
    Self::Triangle,
    Self::Quadratic,
    Self::Cubic,
    Self::Catrom,
    Self::Mitchell,
    Self::Gaussian,
    Self::Sinc,
    Self::Bessel,
    Self::Hanning,
    Self::Hamming,
    Self::Blackman,
    Self::Kaiser,
  ];

  /// Whether the value selects the SDK's own mip chain rather than a resampling kernel.
  pub const fn is_advanced(self) -> bool {
    matches!(self, Self::Advanced)
  }

  /// Editor token for the filter, `tparam_token` (`ETextureParams.cpp`), or the number for one the SDK has no name
  /// for.
  pub fn label(self) -> String {
    match self {
      Self::Box => String::from("Box"),
      Self::Cubic => String::from("Cubic"),
      Self::Point => String::from("Point"),
      Self::Triangle => String::from("Triangle"),
      Self::Quadratic => String::from("Quadratic"),
      Self::Advanced => String::from("Advanced"),
      Self::Catrom => String::from("Catrom"),
      Self::Mitchell => String::from("Mitchell"),
      Self::Gaussian => String::from("Gaussian"),
      Self::Sinc => String::from("Sinc"),
      Self::Bessel => String::from("Bessel"),
      Self::Hanning => String::from("Hanning"),
      Self::Hamming => String::from("Hamming"),
      Self::Blackman => String::from("Blackman"),
      Self::Kaiser => String::from("Kaiser"),
      Self::Unknown(raw) => raw.to_string(),
    }
  }
}

impl Default for ThmMipFilter {
  /// `STextureParams::STextureParams` sets `kMIPFilterBox` (`ETextureParams.h:116`).
  fn default() -> Self {
    Self::Box
  }
}

impl From<u32> for ThmMipFilter {
  fn from(raw: u32) -> Self {
    match raw {
      0 => Self::Box,
      1 => Self::Cubic,
      2 => Self::Point,
      3 => Self::Triangle,
      4 => Self::Quadratic,
      5 => Self::Advanced,
      6 => Self::Catrom,
      7 => Self::Mitchell,
      8 => Self::Gaussian,
      9 => Self::Sinc,
      10 => Self::Bessel,
      11 => Self::Hanning,
      12 => Self::Hamming,
      13 => Self::Blackman,
      14 => Self::Kaiser,
      raw => Self::Unknown(raw),
    }
  }
}

impl From<ThmMipFilter> for u32 {
  fn from(filter: ThmMipFilter) -> Self {
    match filter {
      ThmMipFilter::Box => 0,
      ThmMipFilter::Cubic => 1,
      ThmMipFilter::Point => 2,
      ThmMipFilter::Triangle => 3,
      ThmMipFilter::Quadratic => 4,
      ThmMipFilter::Advanced => 5,
      ThmMipFilter::Catrom => 6,
      ThmMipFilter::Mitchell => 7,
      ThmMipFilter::Gaussian => 8,
      ThmMipFilter::Sinc => 9,
      ThmMipFilter::Bessel => 10,
      ThmMipFilter::Hanning => 11,
      ThmMipFilter::Hamming => 12,
      ThmMipFilter::Blackman => 13,
      ThmMipFilter::Kaiser => 14,
      ThmMipFilter::Unknown(raw) => raw,
    }
  }
}
