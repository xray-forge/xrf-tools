use xrf_chunk::ChunkLayout;
use xrf_editable_level::{EditableDetailHeader, EditableDetailObject, EditableDetailObjectTool};
use xrf_error::{XrfError, XrfResult};
use xrf_level::{DetailModel, LevelDetailsFile, LevelDetailsLight, LevelDetailsSlot};
use xrf_math::{Aabb, Vector3d};
use xrf_utils::narrow_format_size;

use crate::level::build::level_build_details::LevelBuildDetails;
use crate::level::details::level_detail_atlas::LevelDetailAtlas;
use crate::level::details::level_detail_decompile::LevelDetailDecompile;
use crate::level::details::level_detail_library::LevelDetailLibrary;
use crate::level::details::level_detail_match::LevelDetailMatch;
use crate::level::scene::level_scene_texture_file::LevelSceneTextureFile;
use crate::provenance::level_scene_refusal::LevelSceneRefusal;

/// A level's `level.details` back into the detail object tool (`EDetailManager`): each model as the library detail
/// object drawing it, and the slot grid as stored with its light zeroed, as `EDetailManager::Export` leaves it.
pub(crate) struct LevelDetailConversion;

impl LevelDetailConversion {
  /// The objects per square metre the tool takes, `ps_r__Detail_density` as GSC's scenes save it; the compiled level
  /// keeps none, since the slots' densities are already spread.
  pub(crate) const DEFAULT_DENSITY: f32 = 0.3;

  /// The tool of `file`, each model matched in `library`. A model the library lacks is refused and plants nothing,
  /// since `level.details` keeps no texture for it; with no model found there is no tool, as the Build refuses an
  /// empty one.
  ///
  /// Each found object's texture is cut from `atlas` where given, since the game ships the atlas but not the textures.
  /// A texture that does not cut is refused, and a build then needs the library's own.
  ///
  /// # Errors
  ///
  /// Returns an error when more objects are found than the tool's header counts.
  pub(crate) fn decompile(
    file: &LevelDetailsFile,
    library: &LevelDetailLibrary,
    atlas: Option<&LevelDetailAtlas>,
  ) -> XrfResult<LevelDetailDecompile> {
    let mut refused: Vec<LevelSceneRefusal> = Vec::new();
    let mut objects: Vec<EditableDetailObject> = Vec::new();
    let mut found: Vec<LevelDetailMatch<'_>> = Vec::new();
    // Each model's place among the tool's objects, where it was found.
    let places: Vec<Option<u8>> = file
      .objects
      .iter()
      .enumerate()
      .map(|(index, model)| match library.find(model) {
        Some(object) => {
          objects.push(Self::get_object(object.reference, model));
          found.push(object);
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
    let textures: Vec<LevelSceneTextureFile> =
      atlas.map_or_else(Vec::new, |atlas| Self::cut_textures(atlas, &found, &mut refused));

    if objects.is_empty() {
      return Ok(LevelDetailDecompile {
        tool: None,
        textures,
        refused,
      });
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

    Ok(LevelDetailDecompile {
      tool: Some(EditableDetailObjectTool {
        flags: EditableDetailObjectTool::OBJECTS_DRAW,
        header: EditableDetailHeader {
          version: file.header.version,
          object_count: narrow_format_size(objects.len(), "A detail tool's object count")?,
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
        layout: ChunkLayout::default(),
      }),
      textures,
      refused,
    })
  }

  /// Each texture the `found` objects draw, cut once at the first of their places that cuts, since
  /// `EDetailManager::Export` merges each name once; one no place cuts is refused with the first object's reason.
  fn cut_textures(
    atlas: &LevelDetailAtlas,
    found: &[LevelDetailMatch<'_>],
    refused: &mut Vec<LevelSceneRefusal>,
  ) -> Vec<LevelSceneTextureFile> {
    let mut textures: Vec<LevelSceneTextureFile> = Vec::new();

    for (index, first) in found.iter().enumerate() {
      let name: &str = &first.source.texture;
      let is_drawing = |object: &&LevelDetailMatch<'_>| object.source.texture.eq_ignore_ascii_case(name);

      if found[..index].iter().any(|object| is_drawing(&object)) {
        continue;
      }

      let mut first_error: Option<XrfError> = None;

      for object in found[index..].iter().filter(is_drawing) {
        let cut: XrfResult<LevelSceneTextureFile> = object
          .place
          .ok_or_else(|| {
            XrfError::new_invalid_error(format!(
              "The coordinates of object '{}' span no area, so give no place in the atlas",
              object.reference
            ))
          })
          .and_then(|place| atlas.cut(name, &place));

        match cut {
          Ok(texture) => {
            textures.push(texture);
            first_error = None;
            break;
          }
          Err(error) => {
            first_error.get_or_insert(error);
          }
        }
      }

      if let Some(error) = first_error {
        refused.push(LevelSceneRefusal {
          file: String::from(LevelBuildDetails::ATLAS_FILE_NAME),
          object: String::from(name),
          reason: format!("The detail texture was not cut from the atlas, so a build needs the library's own: {error}"),
        });
      }
    }

    textures
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
