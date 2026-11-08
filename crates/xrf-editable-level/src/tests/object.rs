use std::f32::consts::FRAC_PI_2;

use xrf_error::XrfResult;
use xrf_math::{Matrix4x4, Vector2d, Vector3d};

use crate::tests::fixtures::{GUID, particle_part_text, sound_source_part_text};
use crate::{
  EditableLevelValue, EditableObjectPart, EditableObjectTransform, EditableParticleObject, EditableSoundSourceObject,
  EditableTextPart, EditableToolObject,
};

fn read_part<T: EditableToolObject>(text: &str) -> XrfResult<EditableObjectPart<T>> {
  EditableObjectPart::read_from_text_part(&EditableTextPart::read_from_bytes(text.as_bytes())?)
}

fn read_particles(text: &str) -> XrfResult<EditableObjectPart<EditableParticleObject>> {
  read_part(text)
}

fn read_sound_sources(text: &str) -> XrfResult<EditableObjectPart<EditableSoundSourceObject>> {
  read_part(text)
}

fn is_near(first: &Vector3d, second: &Vector3d) -> bool {
  first.distance(second) <= 1e-5
}

#[test]
fn reads_a_particle_part_typed_and_writes_it_back_as_the_editor_saved_it() -> XrfResult {
  let text: String = particle_part_text(&GUID, &[])?;
  let part: EditableObjectPart<EditableParticleObject> = read_particles(&text)?;
  let written: String = String::from_utf8_lossy(&part.to_text_part()?.write_to_bytes()?).into_owned();
  let effect: &EditableParticleObject = &part.objects[0];

  assert_eq!(part.guid, GUID);
  assert_eq!(part.version, 0);
  assert_eq!(part.flags, None, "the particle tool keeps no flags");
  assert_eq!(part.objects.len(), 1);
  assert_eq!(effect.object.name, "industrial_0000");
  assert_eq!(effect.reference, "industrial_particles\\exhaust_isparenie_blue_down");
  assert_eq!(effect.game_types, EditableParticleObject::ALL_GAME_TYPES);
  assert_eq!(effect.object.transform.rotation, Vector3d::new(1.221731, 0.0, 0.0));
  // The editor's `%f` rounds two of the position's last digits otherwise than an exact print of the same floats does;
  // the text reads back to the same part, and is the editor's in everything else.
  assert_eq!(read_particles(&written)?, part);
  assert_eq!(written, text.replace("-25.461156, 70.090608", "-25.461157, 70.090607"));

  Ok(())
}

#[test]
fn takes_an_older_particle_as_playing_in_every_game_mode() -> XrfResult {
  // Version 0x11 predates the game modes, so a stale `game_type` the file still holds is not read.
  let part = read_particles(&particle_part_text(&GUID, &[("version", "17"), ("game_type", "2")])?)?;

  assert_eq!(part.objects[0].game_types, EditableParticleObject::ALL_GAME_TYPES);

  Ok(())
}

#[test]
fn refuses_an_object_of_another_tools_class() -> XrfResult {
  assert!(read_particles(&particle_part_text(&GUID, &[("clsid", "6")])?).is_err());

  Ok(())
}

#[test]
fn reads_a_sound_source_part_typed_and_writes_it_back_as_the_editor_saved_it() -> XrfResult {
  let text: String = sound_source_part_text(&GUID, &[("active_time", "3600.000000, 7200.500000")])?;
  let part: EditableObjectPart<EditableSoundSourceObject> = read_sound_sources(&text)?;
  let sound: &EditableSoundSourceObject = &part.objects[0];
  let written: String = String::from_utf8_lossy(&part.to_text_part()?.write_to_bytes()?).into_owned();

  assert_eq!(part.flags, None, "the sound source tool keeps no flags");
  assert_eq!(sound.object.flags, 2);
  assert_eq!(
    sound.object.transform.position,
    Vector3d::ZERO,
    "the sound stands at its own position"
  );
  assert_eq!(sound.sound, "ambient\\lamp_squeaks_1");
  // Printed through `%d`, uninitialised bits above `flLooped` read as the same unsigned word.
  assert_eq!(sound.flags, 0x9B2D_AAAB);
  assert_eq!(sound.position, Vector3d::new(-79.4942, -8.32556, 57.494_964));
  assert_eq!(sound.min_distance, 1.2);
  assert_eq!(sound.active_time, Vector2d::new(3600.0, 7200.5));
  // The editor's inexact `%f` again: the same floats, printed with other last digits.
  assert_eq!(read_sound_sources(&written)?, part);
  assert_eq!(
    written,
    text.replace("-79.494200, -8.325560, 57.494964", "-79.494202, -8.325560, 57.494965")
  );

  Ok(())
}

#[test]
fn refuses_a_sound_source_the_editor_does_not_load() -> XrfResult {
  for field in [("version", "19"), ("snd_type", "1"), ("active_time", "0.000000")] {
    assert!(
      read_sound_sources(&sound_source_part_text(&GUID, &[field])?).is_err(),
      "{field:?} was read"
    );
  }

  Ok(())
}

#[test]
fn composes_a_transform_scaled_then_turned_then_moved() {
  // A quarter turn about `y` takes the object's `x` axis onto the world's `z`, scaled first.
  let transform: EditableObjectTransform = EditableObjectTransform {
    position: Vector3d::new(10.0, 20.0, 30.0),
    rotation: Vector3d::new(0.0, FRAC_PI_2, 0.0),
    scale: Vector3d::new(2.0, 1.0, 1.0),
  };
  let matrix: Matrix4x4 = transform.to_matrix();

  assert!(is_near(&matrix.to_affine().x_axis, &Vector3d::new(0.0, 0.0, 2.0)));
  assert_eq!(matrix.get_translation(), transform.position);
  assert_eq!(matrix.get_fourth_column(), Matrix4x4::AFFINE_COLUMN);
  assert_eq!(EditableObjectTransform::default().to_matrix(), Matrix4x4::IDENTITY);
}

#[test]
fn decomposes_a_matrix_into_the_transform_it_was_composed_from() -> XrfResult {
  for transform in [
    EditableObjectTransform {
      position: Vector3d::new(-76.429_19, -25.461_156, 70.090_61),
      rotation: Vector3d::new(0.4, -1.2, 2.5),
      scale: Vector3d::new(1.5, 0.5, 2.0),
    },
    // Mirrored on every axis: the scale comes back negative rather than the turn changing.
    EditableObjectTransform {
      position: Vector3d::ZERO,
      rotation: Vector3d::new(0.1, 0.2, 0.3),
      scale: Vector3d::new(-2.0, -2.0, -2.0),
    },
  ] {
    let decomposed: EditableObjectTransform = EditableObjectTransform::from_matrix(&transform.to_matrix())?;

    assert_eq!(decomposed.position, transform.position);
    assert!(is_near(&decomposed.rotation, &transform.rotation));
    assert!(is_near(&decomposed.scale, &transform.scale));
  }

  Ok(())
}

#[test]
fn refuses_a_matrix_no_object_stands_at() {
  let mut projective: Matrix4x4 = Matrix4x4::IDENTITY;
  let collapsed: Matrix4x4 = Matrix4x4::from_scale(&Vector3d::new(1.0, 0.0, 1.0));

  projective.values[3] = 0.5;

  assert!(EditableObjectTransform::from_matrix(&projective).is_err());
  assert!(EditableObjectTransform::from_matrix(&collapsed).is_err());
}

#[test]
fn rounds_a_value_as_its_saved_text_reads_back() {
  let transform: EditableObjectTransform = EditableObjectTransform {
    position: Vector3d::new(1.000_000_4, -0.000_000_1, 0.0),
    rotation: Vector3d::new(1.221_731_2, -0.0, 0.0),
    scale: Vector3d::ONE,
  }
  .round_as_saved();

  assert_eq!(EditableLevelValue::round_as_saved(1.221_731_2), 1.221731);
  assert_eq!(transform.position, Vector3d::new(1.0, 0.0, 0.0));
  assert!(transform.position.y.is_sign_positive(), "a zero saves as `0.000000`");
  assert!(transform.rotation.y.is_sign_positive());
}
