use std::fmt::{Display, Formatter};

use serde::Serialize;

use crate::{LevelGeomVertexBuffer, LevelVertexLayout};

/// The kind of vertex declaration a level stores a visual's geometry in, so two buffers sharing one count once.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LevelGeometryLayoutKind {
  /// Positions alone, the fast path the engine draws depth and shadows from.
  PositionsOnly,
  Tree,
  Lightmapped,
  VertexLit,
  /// A declaration no layout reader accepts.
  Unreadable,
}

impl LevelGeometryLayoutKind {
  pub fn of(buffer: &LevelGeomVertexBuffer) -> Self {
    match LevelVertexLayout::of(buffer) {
      Ok(layout) if layout.is_fastpath() => Self::PositionsOnly,
      Ok(layout) if layout.is_tree() => Self::Tree,
      Ok(layout) if layout.is_lightmapped() => Self::Lightmapped,
      Ok(_) => Self::VertexLit,
      Err(_) => Self::Unreadable,
    }
  }
}

impl Display for LevelGeometryLayoutKind {
  /// The kind as a person reads it on the console.
  fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
    formatter.write_str(match self {
      Self::PositionsOnly => "positions only",
      Self::Tree => "tree",
      Self::Lightmapped => "lightmapped",
      Self::VertexLit => "vertex lit",
      Self::Unreadable => "unreadable",
    })
  }
}
