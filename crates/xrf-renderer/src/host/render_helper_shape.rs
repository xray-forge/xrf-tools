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
}
