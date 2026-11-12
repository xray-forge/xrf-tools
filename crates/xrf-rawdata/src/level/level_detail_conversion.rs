use xrf_editable_level::{EditableDetailHeader, EditableDetailObject, EditableDetailObjectTool};
use xrf_level::{DetailModel, LevelDetailsFile, LevelDetailsLight, LevelDetailsSlot};
use xrf_math::{Aabb, Vector3d};

use crate::level::level_detail_library::LevelDetailLibrary;
use crate::provenance::level_scene_provenance::LevelSceneRefusal;

/// A level's details, `level.details`, back into the detail object tool (`EDetailManager`): each detail model as the
/// library detail object drawing its geometry, and the slot grid as stored, its light zeroed as
/// `EDetailManager::Export` zeroes it before the detail lighter lights it again.
pub(crate) struct LevelDetailConversion;

/// A level's details decompiled: the tool, where any model was found in the library, and the models left out.
pub(crate) struct LevelDetailDecompile {
  pub(crate) tool: Option<EditableDetailObjectTool>,
  pub(crate) refused: Vec<LevelSceneRefusal>,
}

impl LevelDetailConversion {
  /// The objects per square metre the tool takes, `ps_r__Detail_density` as GSC's scenes save it; the compiled level
  /// keeps none, since the slots' densities are already spread.
  pub(crate) const DEFAULT_DENSITY: f32 = 0.3;

  /// The tool of `file`, each model matched in `library`. A model the library lacks is refused and the corners
  /// planting it plant nothing, since `level.details` keeps only its place in the atlas, not its texture; where no
  /// model is found, there is no tool, as the Build refuses one with nothing to merge into its atlas.
  pub(crate) fn decompile(file: &LevelDetailsFile, library: &LevelDetailLibrary) -> LevelDetailDecompile {
    let mut refused: Vec<LevelSceneRefusal> = Vec::new();
    let mut objects: Vec<EditableDetailObject> = Vec::new();
    // Each model's place among the tool's objects, where it was found.
    let places: Vec<Option<u8>> = file
      .objects
      .iter()
      .enumerate()
      .map(|(index, model)| match library.find(model) {
        Some(reference) => {
          objects.push(Self::get_object(reference, model));
          u8::try_from(objects.len() - 1).ok()
        }
        None => {
          refused.push(LevelSceneRefusal {
            file: String::from(LevelDetailsFile::FILE_NAME),
            object: format!("detail object {index}"),
            reason: format!(
              "No library detail object draws its {} vertices and {} triangles; the level keeps only its place in the \
               atlas, not its texture, so its corners plant nothing",
              model.vertices.len(),
              model.get_triangles_count()
            ),
          });
          None
        }
      })
      .collect();

    if objects.is_empty() {
      return LevelDetailDecompile { tool: None, refused };
    }

    let slots: Vec<[u32; EditableDetailObjectTool::SLOT_WORDS]> = file
      .slots
      .iter()
      .map(|slot| {
        (0..LevelDetailsSlot::CORNERS).fold(
          LevelDetailsSlot::with_light(*slot, LevelDetailsLight::UNLIT),
          |slot, corner| match LevelDetailsSlot::get_object(&slot, corner) {
            Some(model) => {
              let place: Option<u8> = places.get(usize::from(model)).copied().flatten();

              LevelDetailsSlot::with_object(slot, corner, place.unwrap_or(LevelDetailsSlot::EMPTY_ID))
            }
            None => slot,
          },
        )
      })
      .collect();

    LevelDetailDecompile {
      tool: Some(EditableDetailObjectTool {
        flags: EditableDetailObjectTool::OBJECTS_DRAW,
        header: EditableDetailHeader {
          version: file.header.version,
          object_count: objects.len() as u32,
          offset_x: file.header.offset_x,
          offset_z: file.header.offset_z,
          size_x: file.header.size_x,
          size_z: file.header.size_z,
        },
        objects,
        color_indices: Vec::new(),
        bbox: Self::get_bounds(file),
        slots,
        base_texture: None,
        density: Self::DEFAULT_DENSITY,
        snap_objects: Vec::new(),
      }),
      refused,
    }
  }

  /// The tool's entry for a model the library's `reference` draws, planted at the model's own scale range and flags.
  fn get_object(reference: &str, model: &DetailModel) -> EditableDetailObject {
    EditableDetailObject::new(reference, model.min_scale, model.max_scale, model.flags)
  }

  /// The box the grid covers: its slots across, from the lowest slot's base to the highest slot's top.
  fn get_bounds(file: &LevelDetailsFile) -> Aabb {
    let size: f32 = xrf_level::DETAIL_SLOT_METERS;
    let (low, high) = file
      .iter_slots()
      .map(|slot: LevelDetailsSlot| (slot.base_height, slot.base_height + slot.height))
      .fold((f32::MAX, f32::MIN), |(low, high), (base, top)| {
        (low.min(base), high.max(top))
      });
    let (low, high) = if low <= high { (low, high) } else { (0.0, 0.0) };

    Aabb {
      min: Vector3d::new(
        -(file.header.offset_x as f32) * size,
        low,
        -(file.header.offset_z as f32) * size,
      ),
      max: Vector3d::new(
        (file.header.size_x as f32 - file.header.offset_x as f32) * size,
        high,
        (file.header.size_z as f32 - file.header.offset_z as f32) * size,
      ),
    }
  }
}
