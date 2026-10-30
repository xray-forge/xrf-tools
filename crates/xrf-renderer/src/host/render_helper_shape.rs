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
  /// A point drawn as three crossed axes `size` metres long: what has a place and no shape.
  Marker {
    center: [f32; 3],
    size: f32,
  },
}
