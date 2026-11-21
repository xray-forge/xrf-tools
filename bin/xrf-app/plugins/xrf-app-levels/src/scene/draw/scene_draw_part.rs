use std::collections::HashMap;
use std::sync::Arc;

use xrf_app_core::session::{SessionPart, SessionSnapshot};
use xrf_editable_object::EditableObjectFile;
use xrf_error::XrfResult;
use xrf_math::{Aabb, Sphere};
use xrf_vfs::XraySearch;
use xrf_visual::convert_declared_bounds;

use crate::scene::{
  SceneDraw, SceneDrawDescription, SceneDrawProblem, SceneDrawProblemKind, SceneDrawnObject, SceneDrawnObjects,
  SceneDrawnObjectsPart, SelectedScene,
};

/// The change event name for a stale scene draw description.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub(crate) const SCENE_DRAW_PART: &str = "sceneDraw";

/// Where a scene's drawn objects stand and which it does not draw, read on first use. It records every `.object` it
/// looked for and read, so one added, removed or changed in the objects roots moves this part and not the scene.
pub(crate) struct SceneDrawPart;

impl SessionPart for SceneDrawPart {
  type Value = Arc<SceneDraw>;

  const NAME: &'static str = SCENE_DRAW_PART;
}

impl SceneDrawPart {
  /// The open scene's draw, read on first use and kept, its library objects read through `objects`, a search of the
  /// scene's objects roots.
  pub(crate) fn get(current: &SessionSnapshot<SelectedScene>, objects: &XraySearch) -> Arc<SceneDraw> {
    current.parts.get_or_record::<Self>(objects, |objects| {
      Arc::new(Self::read(&SceneDrawnObjectsPart::get(current, objects), objects))
    })
  }

  /// The extent and per-object spheres of the scene objects `drawn`, from each `.object`'s mesh boxes placed as
  /// `CSceneObject::GetBox` places them, each library object read once without compiling it; and each object not drawn.
  fn read(drawn: &SceneDrawnObjects, objects: &XraySearch) -> SceneDraw {
    profiling::scope!("scene draw");

    let mut boxes: HashMap<&str, XrfResult<Option<Aabb>>> = HashMap::new();
    let mut bounds: Option<Aabb> = None;
    let mut spheres: HashMap<u32, Sphere> = HashMap::new();
    let mut problems: Vec<SceneDrawProblem> = drawn.problems.clone();

    for object in &drawn.objects {
      let read: &XrfResult<Option<Aabb>> = boxes
        .entry(object.reference.as_str())
        .or_insert_with(|| Self::read_box(objects, object));

      match read {
        Ok(Some(object_box)) => {
          let placed: Aabb = object_box.transform(&object.transform.to_matrix());

          spheres.insert(
            object.id.index,
            convert_declared_bounds(&placed, &placed.get_bounding_sphere()).bounding_sphere,
          );

          match &mut bounds {
            Some(bounds) => bounds.include_box(&placed),
            None => bounds = Some(placed),
          }
        }
        Ok(None) => {}
        Err(error) => problems.push(SceneDrawProblem {
          object: object.id.clone(),
          reference: object.reference.clone(),
          kind: SceneDrawProblemKind::Unreadable,
          message: format!("Library object '{}' does not read: {error}", object.logical_path),
        }),
      }
    }

    problems.sort_by_key(|problem| problem.object.index);

    SceneDraw {
      description: Arc::new(SceneDrawDescription {
        bounds: bounds.map(|bounds| convert_declared_bounds(&bounds, &bounds.get_bounding_sphere())),
        problems,
      }),
      spheres,
    }
  }

  /// The union of the placed library object's mesh boxes, in its own space; none for an object with no mesh.
  fn read_box(objects: &XraySearch, object: &SceneDrawnObject) -> XrfResult<Option<Aabb>> {
    let file: EditableObjectFile = objects
      .read(&object.logical_path)
      .and_then(EditableObjectFile::read_from_bytes)?;

    Ok(
      file
        .meshes
        .iter()
        .flatten()
        .map(|mesh| mesh.bounds)
        .reduce(|mut held, bounds| {
          held.include_box(&bounds);
          held
        }),
    )
  }
}
