use xrf_renderer_core::ShaderStruct;

use crate::scene::static_scene::static_layout::StaticLayout;
use crate::scene::static_scene::static_trace_geometry::StaticTraceGeometry;

/// One geometry of the level's acceleration structure as a traced hit reads it, by its instance's first record and the
/// geometry's index: where its triangles' indices and vertices start, and its surface.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "TraceRecord")]
pub struct StaticTraceRecord {
  pub first_index: u32,
  pub vertex_start: u32,
  pub surface: u32,
  /// `CUT_OUT` where its holes let a ray through; `TREE` or `MODEL` for the arena its vertices are in, the baked one
  /// where neither.
  pub flags: u32,
}

impl StaticTraceRecord {
  pub const CUT_OUT: u32 = 1;
  pub const TREE: u32 = 2;
  pub const MODEL: u32 = 4;
}

impl From<&StaticTraceGeometry> for StaticTraceRecord {
  fn from(geometry: &StaticTraceGeometry) -> Self {
    Self {
      first_index: geometry.first_index,
      vertex_start: geometry.vertex_start,
      surface: geometry.surface,
      flags: if geometry.is_cut_out { Self::CUT_OUT } else { 0 }
        | match geometry.layout {
          StaticLayout::Baked => 0,
          StaticLayout::Tree => Self::TREE,
          StaticLayout::Model => Self::MODEL,
        },
    }
  }
}
