use xrf_editable_level::{EditableObjectTransform, EditableShapeElement, EditableWayPoint};
use xrf_level_source::HelperLinkLines;
use xrf_math::Vector3d;
use xrf_math::glam::{Mat4, Vec3};
use xrf_renderer::{RenderHelperIcon, RenderHelperShape};

use crate::scene::SceneRenderSpace;

/// What one scene object's helper draws and where it stands, in renderer space.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SceneHelperLook {
  /// The transform placing its shapes at the object, sixteen column-major floats.
  pub transform: [f32; 16],
  /// Its shapes in its own renderer space, never empty.
  pub shapes: Vec<RenderHelperShape>,
}

impl SceneHelperLook {
  /// `icon` where `transform` places the object.
  pub(crate) fn icon_at(transform: [f32; 16], icon: RenderHelperIcon) -> Self {
    Self {
      transform,
      shapes: vec![RenderHelperShape::Icon { center: [0.0; 3], icon }],
    }
  }

  /// A shape's `elements`, in its object's own space, where `transform` places them, as `CEditShape` draws them; none
  /// for a shape with no element.
  pub(crate) fn of_elements(transform: &EditableObjectTransform, elements: &[EditableShapeElement]) -> Option<Self> {
    (!elements.is_empty()).then(|| Self {
      transform: SceneRenderSpace::place(transform),
      shapes: elements.iter().map(Self::convert_element).collect(),
    })
  }

  /// The unit box, `-0.5` to `0.5` on each axis, where `transform` places it.
  pub(crate) fn of_unit_box(transform: &EditableObjectTransform) -> Self {
    Self {
      transform: SceneRenderSpace::place(transform),
      shapes: vec![RenderHelperShape::Box {
        axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        center: [0.0; 3],
      }],
    }
  }

  /// A sphere of `radius` metres about the engine space point `center`; none for a radius that is not positive.
  pub(crate) fn of_sphere(center: &Vector3d, radius: f32) -> Option<Self> {
    (radius > 0.0).then(|| Self {
      transform: SceneRenderSpace::place_at(center),
      shapes: vec![RenderHelperShape::Sphere {
        center: [0.0; 3],
        radius,
      }],
    })
  }

  /// A way's engine space `points`, placed at the first: its links as arrowed lines, as the level viewer draws a patrol
  /// path, and `icon` at each point; none for a way with no point.
  pub(crate) fn of_path(points: &[EditableWayPoint], icon: RenderHelperIcon) -> Option<Self> {
    let origin: &Vector3d = &points.first()?.position;
    let start: Vec3 = SceneRenderSpace::convert_point(origin);
    let local: Vec<Vec3> = points
      .iter()
      .map(|point| SceneRenderSpace::convert_point(&point.position) - start)
      .collect();
    let mut ends: Vec<[[f32; 3]; 2]> = Vec::new();

    for (point, from) in points.iter().zip(&local) {
      for link in &point.links {
        if let Some(to) = local.get(link.to) {
          ends.extend(HelperLinkLines::list(*from, *to));
        }
      }
    }

    // A one-point way, as many look and camp paths are, links nothing.
    let mut shapes: Vec<RenderHelperShape> = if ends.is_empty() {
      Vec::new()
    } else {
      vec![RenderHelperShape::Segments { ends }]
    };

    shapes.extend(local.iter().map(|point| RenderHelperShape::Icon {
      center: point.to_array(),
      icon,
    }));

    Some(Self {
      transform: SceneRenderSpace::place_at(origin),
      shapes,
    })
  }

  /// The closed outline of engine space `corners`, placed at their mean; none for fewer than two corners.
  pub(crate) fn of_outline(corners: &[Vector3d]) -> Option<Self> {
    if corners.len() < 2 {
      return None;
    }

    let converted: Vec<Vec3> = corners.iter().map(SceneRenderSpace::convert_point).collect();
    let center: Vec3 = converted.iter().copied().sum::<Vec3>() / converted.len() as f32;
    let ends: Vec<[[f32; 3]; 2]> = converted
      .iter()
      .zip(converted.iter().cycle().skip(1))
      .map(|(from, to)| [(*from - center).to_array(), (*to - center).to_array()])
      .collect();

    Some(Self {
      transform: Mat4::from_translation(center).to_cols_array(),
      shapes: vec![RenderHelperShape::Segments { ends }],
    })
  }

  /// A shape element from its object's engine space into its renderer space: every point and axis mirrored through `z`,
  /// which the transform's mirroring on both sides places where the engine does.
  fn convert_element(element: &EditableShapeElement) -> RenderHelperShape {
    match element {
      EditableShapeElement::Sphere { center, radius } => RenderHelperShape::Sphere {
        center: SceneRenderSpace::convert_point(center).to_array(),
        radius: *radius,
      },
      EditableShapeElement::Box(matrix) => RenderHelperShape::Box {
        axes: [&matrix.x_axis, &matrix.y_axis, &matrix.z_axis]
          .map(|axis| SceneRenderSpace::convert_point(axis).to_array()),
        center: SceneRenderSpace::convert_point(&matrix.translation).to_array(),
      },
    }
  }
}
