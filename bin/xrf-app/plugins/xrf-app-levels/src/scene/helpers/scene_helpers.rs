use std::sync::Arc;

use xrf_editable_level::{
  EditableFogVolumeObject, EditableGlowObject, EditableLightObject, EditableParticleObject, EditablePortalObject,
  EditableShapeObject, EditableSoundEnvironmentObject, EditableSoundSourceObject, EditableSpawnPointObject,
  EditableToolObjects, EditableWayObject,
};
use xrf_math::glam::{Mat4, Vec3};
use xrf_math::{Aabb, Sphere, Vector3d};
use xrf_renderer::{RenderHelperObject, RenderHelperShape, RenderLevelHelpers};
use xrf_visual::VisualBounds;

use crate::scene::{
  SCENE_HELPER_TOOLS, SceneHelpedObject, SceneHelperLook, SceneHelperToolCount, SceneHelperToolEntry,
  SceneHelpersDescription, SceneObjectId, SceneRenderId, SceneRenderSpace, SceneSpawnModelDescription,
  SceneSpawnModels, SceneToolKind, SelectedScene,
};

/// A scene's helpers: the renderer helper of every helped tool object that draws one, each named by its
/// [`SceneRenderId`] and switched by its tool's place, and the page's description of them.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SceneHelpers {
  pub helpers: RenderLevelHelpers,
  pub description: Arc<SceneHelpersDescription>,
}

impl SceneHelpers {
  /// Builds every helped tool's helpers from the parts the open kept, objects in editor load order with group members
  /// first, a spawn point drawn as one of `models` helped beside its model. A tool whose group members do not read
  /// draws none, and one whose own part does not read draws its members alone; the scene's objects report either.
  pub(crate) fn read(scene: &SelectedScene, models: &SceneSpawnModels) -> Self {
    xrf_profiler::scope!("scene helpers");

    let mut objects: Vec<RenderHelperObject> = Vec::new();
    let mut tools: Vec<SceneHelperToolCount> = Vec::new();

    for entry in SCENE_HELPER_TOOLS {
      let read: SceneHelperRead<'_> = SceneHelperRead { scene, entry, models };
      let count: u32 = match entry.tool {
        SceneToolKind::Glow => read.read_tool::<EditableGlowObject>(&mut objects),
        SceneToolKind::Light => read.read_tool::<EditableLightObject>(&mut objects),
        SceneToolKind::Shape => read.read_tool::<EditableShapeObject>(&mut objects),
        SceneToolKind::SoundSource => read.read_tool::<EditableSoundSourceObject>(&mut objects),
        SceneToolKind::Spawn => read.read_tool::<EditableSpawnPointObject>(&mut objects),
        SceneToolKind::Way => read.read_tool::<EditableWayObject>(&mut objects),
        SceneToolKind::Portal => read.read_tool::<EditablePortalObject>(&mut objects),
        SceneToolKind::SoundEnvironment => read.read_tool::<EditableSoundEnvironmentObject>(&mut objects),
        SceneToolKind::Particle => read.read_tool::<EditableParticleObject>(&mut objects),
        SceneToolKind::FogVolume => read.read_tool::<EditableFogVolumeObject>(&mut objects),
        // Drawn as models, or as what the editor's helpers do not picture yet.
        SceneToolKind::Group
        | SceneToolKind::SceneObject
        | SceneToolKind::Sector
        | SceneToolKind::DetailObject
        | SceneToolKind::AiMap
        | SceneToolKind::Wallmark => 0,
      };

      if count > 0 {
        tools.push(SceneHelperToolCount {
          tool: entry.tool,
          count,
        });
      }
    }

    // A spawn point drawn only as its model still counts in the framed extent, at its position.
    let modelled: Vec<Vec3> = models
      .objects
      .iter()
      .map(|model| SceneRenderSpace::convert_point(&model.transform.position))
      .collect();

    Self {
      description: Arc::new(SceneHelpersDescription {
        bounds: Self::bound(&objects, &modelled),
        tools,
        spawn_models: models.objects.iter().map(SceneSpawnModelDescription::from).collect(),
      }),
      helpers: RenderLevelHelpers { objects },
    }
  }

  /// The sphere about `object`'s helper extent, in renderer space, bounded as the scene's extent is; none for an object
  /// that draws no helper.
  pub(crate) fn find_sphere(&self, object: &SceneObjectId) -> Option<Sphere> {
    let id: SceneRenderId = SceneRenderId::of(object)?;
    // Ids sort by tool, then by index, and the helpers are read in that order.
    let found: usize = self
      .helpers
      .objects
      .binary_search_by_key(&id.0, |helper| helper.id)
      .ok()?;

    Self::bound(&self.helpers.objects[found..=found], &[]).map(|bounds| bounds.bounding_sphere)
  }

  /// The box holding every placed point the helpers draw through, spheres by their boxes, and the renderer space
  /// `extra` points; none for no helper and no point.
  fn bound(objects: &[RenderHelperObject], extra: &[Vec3]) -> Option<VisualBounds> {
    let mut points: Vec<Vector3d> = extra.iter().map(Vector3d::from).collect();

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

          Vector3d::from(placed)
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

/// What reading one helped tool's objects into helpers needs: the scene, the tool's row, and the spawn models.
struct SceneHelperRead<'a> {
  scene: &'a SelectedScene,
  entry: &'a SceneHelperToolEntry,
  models: &'a SceneSpawnModels,
}

impl SceneHelperRead<'_> {
  /// Adds the helpers of the tool's `O` objects to `objects` and returns how many objects are helped, counting one
  /// drawn only as its model.
  fn read_tool<O: SceneHelpedObject>(&self, objects: &mut Vec<RenderHelperObject>) -> u32 {
    let Self { scene, entry, models } = self;
    // The scene's objects report a part that does not read; its groups' members are helped still.
    let Ok((read, _)) = EditableToolObjects::<O>::read_keeping_members(&scene.scene, scene.guid) else {
      return 0;
    };
    let mut count: u32 = 0;

    for (index, object) in read.iter_indexed() {
      let Some(id) = SceneRenderId::of(&SceneObjectId {
        tool: entry.tool,
        index,
      }) else {
        continue;
      };
      let is_modelled: bool =
        entry.tool == SceneToolKind::Spawn && models.find(index).is_some_and(|model| model.is_drawn);
      let look: Option<SceneHelperLook> = if is_modelled {
        object.get_modelled_look()
      } else {
        object.get_look()
      };

      if is_modelled || look.is_some() {
        count += 1;
      }

      let Some(SceneHelperLook { transform, shapes }) = look else {
        continue;
      };

      objects.push(RenderHelperObject {
        id: id.0,
        kind: entry.kind,
        color: entry.get_rgb(),
        draw_distance: entry.draw_distance,
        transform,
        shapes,
        motion: None,
      });
    }

    count
  }
}
