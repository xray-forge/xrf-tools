use wgpu::util::DeviceExt;

use crate::contract::render_overlay::RenderOverlay;

/// A viewport's overlays as their vertices: every line segment's two ends, and the sun's disc, uploaded once a set.
pub struct LevelOverlays {
  /// The set they were built from.
  pub version: u64,
  /// Position, then colour with one in `w` where depth tests it, a vertex; and how many vertices.
  pub lines: Option<(wgpu::Buffer, u32)>,
  /// Colour, then size in pixels.
  pub sun: Option<wgpu::Buffer>,
}

impl LevelOverlays {
  pub fn new(device: &wgpu::Device, overlays: &[RenderOverlay], version: u64) -> Self {
    let mut vertices: Vec<[f32; 7]> = Vec::new();
    let mut sun: Option<[f32; 4]> = None;

    for overlay in overlays {
      match overlay {
        RenderOverlay::Lines {
          positions,
          colors,
          is_depth_tested,
        } => {
          let tested: f32 = f32::from(u8::from(*is_depth_tested));
          let ends: &[[f32; 3]] = positions.as_chunks::<3>().0;
          let tints: &[[f32; 3]] = colors.as_chunks::<3>().0;
          // A segment needs both its ends, so one left without its pair is dropped with the set it came in.
          let count: usize = ends.len().min(tints.len()) / 2 * 2;

          for (position, color) in ends.iter().zip(tints).take(count) {
            vertices.push([
              position[0],
              position[1],
              position[2],
              color[0],
              color[1],
              color[2],
              tested,
            ]);
          }
        }
        RenderOverlay::Sun { color, size } => sun = Some([color[0], color[1], color[2], *size]),
      }
    }

    Self {
      version,
      lines: (!vertices.is_empty()).then(|| {
        (
          device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("overlay lines"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
          }),
          vertices.len() as u32,
        )
      }),
      sun: sun.map(|sun| {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
          label: Some("overlay sun"),
          contents: bytemuck::cast_slice(&sun),
          usage: wgpu::BufferUsages::VERTEX,
        })
      }),
    }
  }
}
