use std::fmt;

use xrf_error::XrfError;

/// Why a window of a mesh's indices cannot be drawn, told the same way by every packer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum VisualIndexFault {
  /// The window reaches past the indices it slides over.
  Outside { offset: u32, count: u32, available: u32 },
  /// The window starts inside a triangle rather than on one.
  Misaligned { offset: u32 },
  /// The window starts further along a buffer than 32 bits address.
  Unaddressable { base: u32, offset: u32 },
  /// One of the window's indices names a vertex the mesh does not have.
  Stray { index: u16, vertex_count: u32 },
}

impl fmt::Display for VisualIndexFault {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Outside {
        offset,
        count,
        available,
      } => write!(
        formatter,
        "draws {count} indices from offset {offset}, past the {available} the index chunk holds"
      ),
      Self::Misaligned { offset } => write!(formatter, "draws from offset {offset}, which starts no triangle"),
      Self::Unaddressable { base, offset } => write!(
        formatter,
        "draws from offset {offset} of indices stored from {base}, past what a buffer addresses"
      ),
      Self::Stray { index, vertex_count } => write!(
        formatter,
        "references vertex {index}, past the {vertex_count} the vertex chunk holds"
      ),
    }
  }
}

impl From<VisualIndexFault> for XrfError {
  fn from(fault: VisualIndexFault) -> Self {
    Self::new_invalid_error(fault.to_string())
  }
}
