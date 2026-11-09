use xrf_editable_level::{EditableWallmark, EditableWallmarkSlot, EditableWallmarkTool, EditableWallmarkVertex};
use xrf_error::{XrfError, XrfResult};
use xrf_level::{LevelWallmark, LevelWallmarkSlot, LevelWallmarkVertex, LevelWallmarksFile};
use xrf_math::Sphere;

/// A level's baked decals, between the scene's `wallmark.part` and the compiled `level.wallmarks`, which the build
/// writes through `ESceneWallmarkTool::Export` (`ESceneWallmarkTools.cpp:549`).
pub struct LevelWallmarkConversion;

impl LevelWallmarkConversion {
  /// The wall mark tool a `level.wallmarks` was exported from: a slot per stored slot, each decal with the sphere the
  /// file keeps, the box `AddWallmark` gives its fan, no selection (as every decal GSC saved), and the size and turn of
  /// the brush, a new tool's, since the file keeps neither.
  ///
  /// # Errors
  ///
  /// Returns an error naming a decal of no vertex, which has no box.
  pub fn decompile(file: &LevelWallmarksFile) -> XrfResult<EditableWallmarkTool> {
    let tool: EditableWallmarkTool = EditableWallmarkTool::new(Vec::new());
    let slots: Vec<EditableWallmarkSlot> = file
      .slots
      .iter()
      .enumerate()
      .map(|(slot_index, slot)| {
        let marks: Vec<EditableWallmark> = slot
          .marks
          .iter()
          .enumerate()
          .map(|(index, mark)| {
            let vertices: Vec<EditableWallmarkVertex> = mark
              .vertices
              .iter()
              .map(|vertex| EditableWallmarkVertex {
                position: vertex.position,
                color: vertex.color,
                u: vertex.u,
                v: vertex.v,
              })
              .collect();
            let (bbox, _) = EditableWallmark::get_bounds(&vertices).ok_or_else(|| {
              XrfError::new_invalid_error(format!("Wall mark {index} of slot {slot_index} has no vertex"))
            })?;

            Ok(EditableWallmark {
              flags: 0,
              bbox,
              bounds: Sphere {
                center: mark.center,
                radius: mark.radius,
              },
              width: tool.mark_width,
              height: tool.mark_height,
              rotation: tool.mark_rotation,
              vertices,
            })
          })
          .collect::<XrfResult<Vec<EditableWallmark>>>()?;

        Ok(EditableWallmarkSlot {
          shader: slot.shader.clone(),
          texture: slot.texture.clone(),
          marks,
        })
      })
      .collect::<XrfResult<Vec<EditableWallmarkSlot>>>()?;

    Ok(EditableWallmarkTool { slots, ..tool })
  }

  /// The `level.wallmarks` the build writes from the tool as the editor loaded it: a decal of more than
  /// [`EditableWallmark::MAX_VERTICES`] dropped (`LoadStream`), then each slot left empty dropped (`RefiningSlots`),
  /// and each decal's sphere and fan.
  pub fn export(tool: &EditableWallmarkTool) -> LevelWallmarksFile {
    LevelWallmarksFile {
      slots: tool
        .slots
        .iter()
        .filter_map(|slot| {
          let marks: Vec<LevelWallmark> = slot
            .marks
            .iter()
            .filter(|mark| mark.vertices.len() <= EditableWallmark::MAX_VERTICES)
            .map(|mark| LevelWallmark {
              center: mark.bounds.center,
              radius: mark.bounds.radius,
              vertices: mark
                .vertices
                .iter()
                .map(|vertex| LevelWallmarkVertex {
                  position: vertex.position,
                  color: vertex.color,
                  u: vertex.u,
                  v: vertex.v,
                })
                .collect(),
            })
            .collect();

          (!marks.is_empty()).then(|| LevelWallmarkSlot {
            shader: slot.shader.clone(),
            texture: slot.texture.clone(),
            marks,
          })
        })
        .collect(),
    }
  }
}
