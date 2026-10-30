use glam::Vec3;
use wgpu::util::DeviceExt;

use crate::contract::render_helper_options::RenderHelperOptions;
use crate::scene::level::feature::view_feature::ViewFeature;
use crate::scene::level::helpers::helper_vertex::HelperVertex;
use crate::scene::level::helpers::level_helper_object::LevelHelperObject;
use crate::scene::level::helpers::level_helpers::LevelHelpers;

/// What a view draws of its scene's helpers: the wires its options keep, a line list built again only when the scene's
/// set or the options change, which the window pass draws and a pick draws again wide.
#[derive(Default)]
pub struct HelpersFeature {
  /// The set and the options the wires were built for.
  built: Option<(u64, RenderHelperOptions)>,
  /// Two vertices a segment, and how many vertices.
  wires: Option<(wgpu::Buffer, u32)>,
}

impl HelpersFeature {
  /// Builds the wires again where the scene's helpers or the view's options moved since: the objects of the kinds
  /// switched on and not hidden, none while helpers are off.
  pub fn set(&mut self, device: &wgpu::Device, helpers: &LevelHelpers, options: &RenderHelperOptions) {
    if self
      .built
      .as_ref()
      .is_some_and(|(version, built)| *version == helpers.version && built == options)
    {
      return;
    }

    self.built = Some((helpers.version, options.clone()));

    let vertices: Vec<HelperVertex> = Self::to_vertices(helpers, options);

    self.wires = (!vertices.is_empty()).then(|| {
      (
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
          label: Some("helper wires"),
          contents: bytemuck::cast_slice(&vertices),
          usage: wgpu::BufferUsages::VERTEX,
        }),
        vertices.len() as u32,
      )
    });
  }

  /// The wires to draw, and how many vertices; none where nothing is drawn.
  pub fn get_wires(&self) -> Option<(&wgpu::Buffer, u32)> {
    self.wires.as_ref().map(|(buffer, count)| (buffer, *count))
  }

  /// The ends of every wire `options` keeps of `helpers`.
  pub fn to_vertices(helpers: &LevelHelpers, options: &RenderHelperOptions) -> Vec<HelperVertex> {
    if !options.is_shown {
      return Vec::new();
    }

    helpers
      .objects
      .iter()
      .filter(|object| {
        object.kind < u32::BITS
          && options.kinds & 1 << object.kind != 0
          && options.hidden.binary_search(&object.id).is_err()
      })
      .flat_map(|object: &LevelHelperObject| {
        let is_selected: bool = options.selected.binary_search(&object.id).is_ok();
        let color: [f32; 4] = [
          object.color[0],
          object.color[1],
          object.color[2],
          f32::from(u8::from(is_selected)),
        ];
        let fade: [f32; 2] = [
          if is_selected {
            0.0
          } else {
            object.draw_distance * options.distance_scale
          },
          options.occluded_strength,
        ];

        helpers.segments[object.segments.clone()]
          .iter()
          .flatten()
          .map(move |end: &Vec3| HelperVertex {
            position: end.to_array(),
            color,
            fade,
            object: object.id,
          })
      })
      .collect()
  }
}

impl ViewFeature for HelpersFeature {
  /// The wires' vertices.
  fn get_memory(&self) -> u64 {
    self.wires.as_ref().map_or(0, |(buffer, _)| buffer.size())
  }

  fn release(&mut self) {
    self.built = None;
    self.wires = None;
  }
}
