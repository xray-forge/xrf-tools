use serde::{Deserialize, Serialize};

use crate::contract::render_surface_span::RenderSurfaceSpan;

/// How much geometry one shader table entry of a viewport's level draws, across the sectors resident.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderSurfaceGeometry {
  pub shader_id: u16,
  /// Drawables of the level's visuals naming the entry.
  pub drawables: u32,
  /// Triangles drawn at whole detail, a mesh's once for each place it stands in.
  pub triangles: u32,
  /// What its base coordinate covers over every draw together.
  pub span: Option<RenderSurfaceSpan>,
  /// The narrowest range any single draw covers.
  pub narrowest: Option<RenderSurfaceSpan>,
}

impl RenderSurfaceGeometry {
  pub fn new(shader_id: u16) -> Self {
    Self {
      shader_id,
      drawables: 0,
      triangles: 0,
      span: None,
      narrowest: None,
    }
  }

  /// Counts one draw's drawables, triangles and the range its coordinate covers in.
  pub fn add(&mut self, drawables: u32, triangles: u32, drawn: Option<RenderSurfaceSpan>) {
    self.merge(&Self {
      shader_id: self.shader_id,
      drawables,
      triangles,
      span: drawn,
      narrowest: drawn,
    });
  }

  /// Counts what another measure of the same entry counted.
  pub fn merge(&mut self, other: &Self) {
    self.drawables = self.drawables.saturating_add(other.drawables);
    self.triangles = self.triangles.saturating_add(other.triangles);
    self.span = merge_spans(self.span, other.span, RenderSurfaceSpan::merge);
    self.narrowest = merge_spans(self.narrowest, other.narrowest, |held, drawn| {
      if drawn.get_area() < held.get_area() {
        drawn
      } else {
        held
      }
    });
  }
}

fn merge_spans(
  held: Option<RenderSurfaceSpan>,
  drawn: Option<RenderSurfaceSpan>,
  combine: impl Fn(RenderSurfaceSpan, RenderSurfaceSpan) -> RenderSurfaceSpan,
) -> Option<RenderSurfaceSpan> {
  match (held, drawn) {
    (Some(held), Some(drawn)) => Some(combine(held, drawn)),
    (held, drawn) => held.or(drawn),
  }
}
