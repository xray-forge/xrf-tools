use std::sync::Arc;

use glam::{Mat4, Vec3};
use xrf_editable_level::{
  EditableFogVolumeObject, EditableGlowObject, EditableLightObject, EditableParticleObject, EditablePortalObject,
  EditableShapeObject, EditableSoundEnvironmentObject, EditableSoundSourceObject, EditableSpawnPointObject,
  EditableWayObject,
};
use xrf_math::{Aabb, Vector3d};
use xrf_rawdata::LevelSceneToolObjects;
use xrf_renderer::{RenderHelperObject, RenderHelperShape, RenderLevelHelpers};
use xrf_visual::VisualBounds;

use crate::scene::{
  SCENE_HELPER_TOOLS, SceneHelpedObject, SceneHelperId, SceneHelperLook, SceneHelperToolCount, SceneHelperToolEntry,
  SceneHelpersDescription, SceneObjectId, SceneToolKind, SelectedScene,
};

/// A scene's helpers: the renderer's helper of every object of a helped tool that draws one, each named by its
/// [`SceneHelperId`] and switched by its tool's place, and what the page is told of them.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SceneHelpers {
  pub helpers: RenderLevelHelpers,
  pub description: Arc<SceneHelpersDescription>,
}

impl SceneHelpers {
  /// Every helped tool's objects as the editor loads them, the groups' members first, each helped in its tool's look,
  /// read from the parts the open kept and nothing else; a tool whose objects do not read draws none, which the scene's
  /// objects report.
  pub(crate) fn read(scene: &SelectedScene) -> Self {
    profiling::scope!("scene helpers");

    let mut objects: Vec<RenderHelperObject> = Vec::new();
    let mut tools: Vec<SceneHelperToolCount> = Vec::new();

    for entry in SCENE_HELPER_TOOLS {
      let before: usize = objects.len();

      match entry.tool {
        SceneToolKind::Glow => Self::read_tool::<EditableGlowObject>(scene, entry, &mut objects),
        SceneToolKind::Light => Self::read_tool::<EditableLightObject>(scene, entry, &mut objects),
        SceneToolKind::Shape => Self::read_tool::<EditableShapeObject>(scene, entry, &mut objects),
        SceneToolKind::SoundSource => Self::read_tool::<EditableSoundSourceObject>(scene, entry, &mut objects),
        SceneToolKind::Spawn => Self::read_tool::<EditableSpawnPointObject>(scene, entry, &mut objects),
        SceneToolKind::Way => Self::read_tool::<EditableWayObject>(scene, entry, &mut objects),
        SceneToolKind::Portal => Self::read_tool::<EditablePortalObject>(scene, entry, &mut objects),
        SceneToolKind::SoundEnvironment => {
          Self::read_tool::<EditableSoundEnvironmentObject>(scene, entry, &mut objects);
        }
        SceneToolKind::Particle => Self::read_tool::<EditableParticleObject>(scene, entry, &mut objects),
        SceneToolKind::FogVolume => Self::read_tool::<EditableFogVolumeObject>(scene, entry, &mut objects),
        // Drawn as models, or as what the editor's helpers do not picture yet.
        SceneToolKind::Group
        | SceneToolKind::SceneObject
        | SceneToolKind::Sector
        | SceneToolKind::DetailObject
        | SceneToolKind::AiMap
        | SceneToolKind::Wallmark => {}
      }

      if objects.len() > before {
        tools.push(SceneHelperToolCount {
          tool: entry.tool,
          count: (objects.len() - before) as u32,
        });
      }
    }

    Self {
      description: Arc::new(SceneHelpersDescription {
        bounds: Self::bound(&objects),
        tools,
      }),
      helpers: RenderLevelHelpers { objects },
    }
  }

  /// The helpers of the tool `entry` heads, an object of type `O`, added to `objects`.
  fn read_tool<O: SceneHelpedObject>(
    scene: &SelectedScene,
    entry: &SceneHelperToolEntry,
    objects: &mut Vec<RenderHelperObject>,
  ) {
    let Ok(read) = LevelSceneToolObjects::<O>::read(&scene.scene, scene.guid) else {
      return;
    };

    for (object, index) in read
      .members
      .iter()
      .map(|member| &member.object)
      .chain(&read.part.objects)
      .zip(0u32..)
    {
      let (Some(SceneHelperLook { transform, shapes }), Some(id)) = (
        object.get_look(),
        SceneHelperId::of(&SceneObjectId {
          tool: entry.tool,
          index,
        }),
      ) else {
        continue;
      };

      objects.push(RenderHelperObject {
        id: id.0,
        kind: entry.kind,
        color: entry.get_rgb(),
        draw_distance: entry.draw_distance,
        transform,
        shapes,
      });
    }
  }

  /// The box holding every point each helper draws through, its spheres by their boxes, where it stands; none for no
  /// helper.
  fn bound(objects: &[RenderHelperObject]) -> Option<VisualBounds> {
    let mut points: Vec<Vector3d> = Vec::new();

    for object in objects {
      let transform: Mat4 = Mat4::from_cols_array(&object.transform);

      for shape in &object.shapes {
        let reached: Vec<Vec3> = match shape {
          RenderHelperShape::Sphere { center, radius } => Self::list_corners(
            Vec3::from_array(*center),
            [Vec3::X, Vec3::Y, Vec3::Z].map(|axis| axis * *radius),
          )
          .to_vec(),
          RenderHelperShape::Box { axes, center } => {
            Self::list_corners(Vec3::from_array(*center), axes.map(|axis| Vec3::from_array(axis) * 0.5)).to_vec()
          }
          RenderHelperShape::Segments { ends } => ends.iter().flatten().copied().map(Vec3::from_array).collect(),
          RenderHelperShape::Icon { center, .. } => vec![Vec3::from_array(*center)],
          RenderHelperShape::Model { .. } => Vec::new(),
        };

        points.extend(reached.into_iter().map(|point| {
          let placed: Vec3 = transform.transform_point3(point);

          Vector3d::new(placed.x, placed.y, placed.z)
        }));
      }
    }

    Aabb::from_points(&points).map(|bounding_box: Aabb| VisualBounds {
      bounding_sphere: bounding_box.get_bounding_sphere(),
      bounding_box,
    })
  }

  /// The eight corners of the box about `center` reaching each of `half_axes` either way.
  fn list_corners(center: Vec3, half_axes: [Vec3; 3]) -> [Vec3; 8] {
    let [i, j, k] = half_axes;

    std::array::from_fn(|corner: usize| {
      let [x, y, z] = [corner & 1, (corner >> 1) & 1, (corner >> 2) & 1].map(|bit| if bit == 0 { -1.0 } else { 1.0 });

      center + i * x + j * y + k * z
    })
  }
}
