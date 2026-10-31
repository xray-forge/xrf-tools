use crate::contract::render_helper_icon::RenderHelperIcon;

/// One shape of a helper in its object's own space, renderer space.
#[derive(Clone, Debug, PartialEq)]
pub enum RenderHelperShape {
  Sphere {
    center: [f32; 3],
    radius: f32,
  },
  /// The unit cube about the origin taken through `axes` and moved to `center`.
  Box {
    axes: [[f32; 3]; 3],
    center: [f32; 3],
  },
  /// Wires as they stand, each by its two ends: a path's links.
  Segments {
    ends: Vec<[[f32; 3]; 2]>,
  },
  /// A spawned object's drawn model, by its index among the level's spawned objects: where it shows, tinted and
  /// outlined in its helper's colour, and its box while it is hovered or selected.
  Model {
    object: u32,
  },
  /// A picture standing at a point, drawn at a constant size on screen: what has a place and no shape.
  Icon {
    center: [f32; 3],
    icon: RenderHelperIcon,
  },
}
