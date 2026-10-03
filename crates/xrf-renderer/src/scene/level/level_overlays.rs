use glam::Vec3;
use wgpu::util::DeviceExt;

use crate::contract::render_overlay::RenderOverlay;

/// A viewport's overlays as their vertices: every line segment's two ends, the points, and the sun's disc, uploaded
/// once a set; and the skeleton's segments, uploaded again every frame its pose may have moved.
pub struct LevelOverlays {
  /// The set they were built from.
  pub version: u64,
  /// Position, then colour with one in `w` where depth tests it, a vertex; and how many vertices.
  pub lines: Option<(wgpu::Buffer, u32)>,
  /// Position and size, then colour with one in `w` where depth tests it, a point; and how many points.
  pub points: Option<(wgpu::Buffer, u32)>,
  /// Colour, then size in pixels.
  pub sun: Option<wgpu::Buffer>,
  /// The skeleton's colour and whether depth tests it, while the set draws one.
  pub skeleton: Option<([f32; 3], bool)>,
  /// The skeleton's segments as `lines` holds its own, this frame.
  pub skeleton_lines: Option<(wgpu::Buffer, u32)>,
}

impl LevelOverlays {
  pub fn new(device: &wgpu::Device, overlays: &[RenderOverlay], version: u64) -> Self {
    let mut vertices: Vec<[f32; 7]> = Vec::new();
    let mut points: Vec<[f32; 8]> = Vec::new();
    let mut sun: Option<[f32; 4]> = None;
    let mut skeleton: Option<([f32; 3], bool)> = None;

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
        RenderOverlay::Points {
          positions,
          color,
          size,
          is_depth_tested,
        } => {
          let tested: f32 = f32::from(u8::from(*is_depth_tested));

          points.extend(
            positions
              .as_chunks::<3>()
              .0
              .iter()
              .map(|it| [it[0], it[1], it[2], *size, color[0], color[1], color[2], tested]),
          );
        }
        RenderOverlay::Sun { color, size } => sun = Some([color[0], color[1], color[2], *size]),
        RenderOverlay::Skeleton { color, is_depth_tested } => skeleton = Some((*color, *is_depth_tested)),
      }
    }

    Self {
      version,
      lines: create_vertices(device, "overlay lines", &vertices),
      points: create_vertices(device, "overlay points", &points),
      sun: sun.map(|sun| {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
          label: Some("overlay sun"),
          contents: bytemuck::cast_slice(&sun),
          usage: wgpu::BufferUsages::VERTEX,
        })
      }),
      skeleton,
      skeleton_lines: None,
    }
  }

  /// Takes this frame's skeleton segments, child then parent, in renderer space, while the set draws a skeleton.
  pub fn set_skeleton(&mut self, device: &wgpu::Device, segments: &[(Vec3, Vec3)]) {
    let Some((color, is_depth_tested)) = self.skeleton else {
      return;
    };
    let tested: f32 = f32::from(u8::from(is_depth_tested));
    let vertices: Vec<[f32; 7]> = segments
      .iter()
      .flat_map(|(child, parent)| [*child, *parent])
      .map(|end| [end.x, end.y, end.z, color[0], color[1], color[2], tested])
      .collect();

    self.skeleton_lines = create_vertices(device, "overlay skeleton", &vertices);
  }
}

/// A vertex buffer of what is given and how many, or none for nothing.
fn create_vertices<T: bytemuck::Pod>(
  device: &wgpu::Device,
  label: &str,
  vertices: &[T],
) -> Option<(wgpu::Buffer, u32)> {
  (!vertices.is_empty()).then(|| {
    (
      device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(vertices),
        usage: wgpu::BufferUsages::VERTEX,
      }),
      vertices.len() as u32,
    )
  })
}
