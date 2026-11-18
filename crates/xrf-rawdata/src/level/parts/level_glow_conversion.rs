use xrf_editable_level::{EditableCustomObject, EditableGlowObject, EditableObjectPart, EditableObjectTransform};
use xrf_error::{XrfError, XrfResult};
use xrf_level::{LevelFile, LevelShaderEntry};
use xrf_math::Vector3d;

use crate::level::level_decompile_context::LevelDecompileContext;

/// A level's light flares, from the compiled `fsL_GLOWS` back into the scene's `glow.part`. The level compiler writes
/// each `b_glow` the build gave it (`SceneBuilder::BuildGlow`, `BuilderRemote.cpp:1161`) as its position, radius and
/// the shader table row of its shader and texture; the fixed-size flag does not survive.
pub struct LevelGlowConversion;

impl LevelGlowConversion {
  /// The glow part of a compiled level: a glow per `fsL_GLOWS` record, unturned at its position, named after its
  /// texture, its shader and texture those of its shader table row, its flags clear.
  ///
  /// # Errors
  ///
  /// Returns an error naming the glow whose shader row is missing, or not a shader with a texture.
  pub fn decompile(
    level: &LevelFile,
    context: &mut LevelDecompileContext,
  ) -> XrfResult<EditableObjectPart<EditableGlowObject>> {
    let shaders: &[LevelShaderEntry] = level.shaders.as_ref().map_or(&[], |chunk| &chunk.entries);
    let objects: Vec<EditableGlowObject> = level
      .get_glows()
      .iter()
      .enumerate()
      .map(|(index, glow)| {
        let (shader, texture) = match shaders.get(usize::from(glow.shader)) {
          Some(LevelShaderEntry::Reference(reference)) if !reference.textures.is_empty() => {
            (reference.shader.clone(), reference.textures[0].clone())
          }
          _ => {
            return Err(XrfError::new_invalid_error(format!(
              "Glow {index} draws with shader row {} of {}, which is no shader with a texture",
              glow.shader,
              shaders.len()
            )));
          }
        };

        Ok(EditableGlowObject {
          object: EditableCustomObject::new(
            context.names.get_next_after_file(&texture),
            EditableObjectTransform {
              position: glow.position,
              rotation: Vector3d::ZERO,
              scale: Vector3d::new(1.0, 1.0, 1.0),
            },
          ),
          radius: glow.radius,
          shader,
          texture,
          flags: 0,
        })
      })
      .collect::<XrfResult<Vec<EditableGlowObject>>>()?;

    Ok(EditableObjectPart::new(context.guid, objects))
  }
}
