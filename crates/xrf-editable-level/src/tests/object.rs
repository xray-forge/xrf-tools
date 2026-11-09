use std::f32::consts::FRAC_PI_2;

use xrf_error::XrfResult;
use xrf_math::{Affine3d, Matrix4x4, Vector2d, Vector3d};

use crate::tests::fixtures::{
  GUID, fog_volume_part_text, particle_part_text, sound_environment_part_text, sound_source_part_text,
};
use crate::{
  EditableFogVolumeKind, EditableFogVolumeObject, EditableLevelValue, EditableObjectPart, EditableObjectTransform,
  EditableParticleObject, EditableShapeElement, EditableSoundEnvironmentObject, EditableSoundSourceObject,
  EditableTextPart, EditableToolObject,
};

fn read_part<T: EditableToolObject>(text: &str) -> XrfResult<EditableObjectPart<T>> {
  EditableObjectPart::read_from_text_part(&EditableTextPart::read_from_bytes(text.as_bytes())?)
}

fn read_fog_volumes(text: &str) -> XrfResult<EditableObjectPart<EditableFogVolumeObject>> {
  read_part(text)
}

fn read_particles(text: &str) -> XrfResult<EditableObjectPart<EditableParticleObject>> {
  read_part(text)
}

fn read_sound_sources(text: &str) -> XrfResult<EditableObjectPart<EditableSoundSourceObject>> {
  read_part(text)
}

fn read_sound_environments(text: &str) -> XrfResult<EditableObjectPart<EditableSoundEnvironmentObject>> {
  read_part(text)
}

#[test]
fn reads_a_fog_volume_part_typed_and_writes_it_back_as_the_editor_saved_it() -> XrfResult {
  let text: String = fog_volume_part_text(&GUID, &[])?;
  let part: EditableObjectPart<EditableFogVolumeObject> = read_fog_volumes(&text)?;
  let volume: &EditableFogVolumeObject = &part.objects[0];

  assert_eq!(
    volume.group,
    EditableFogVolumeObject::UNGROUPED,
    "`u32(-1)` printed through `%d`"
  );
  assert_eq!(volume.get_profile(), Some(EditableFogVolumeObject::DEFAULT_PROFILE));
  assert_eq!(String::from_utf8_lossy(&part.to_text_part()?.write_to_bytes()?), text);

  Ok(())
}

#[test]
fn writes_no_profile_for_an_occlusion_volume() -> XrfResult {
  let text: String = fog_volume_part_text(&GUID, &[("folume_type", "1"), ("group_id", "3")])?;
  let part: EditableObjectPart<EditableFogVolumeObject> = read_fog_volumes(&text)?;
  let written: String = String::from_utf8_lossy(&part.to_text_part()?.write_to_bytes()?).into_owned();

  assert_eq!(part.objects[0].kind, EditableFogVolumeKind::Occlusion);
  assert_eq!(part.objects[0].group, 3);
  assert!(!written.contains("profile"), "{written}");

  Ok(())
}

#[test]
fn takes_an_older_fog_volume_as_the_editor_does() -> XrfResult {
  // Version 1 keeps no profile and version 0 no type or group: an ungrouped emitter of the default profile.
  let older = read_fog_volumes(&fog_volume_part_text(
    &GUID,
    &[("version", "1"), ("profile", "environment\\fog\\swamp.ltx")],
  )?)?;
  let oldest = read_fog_volumes(&fog_volume_part_text(&GUID, &[("version", "0"), ("group_id", "4")])?)?;

  assert_eq!(
    older.objects[0].get_profile(),
    Some(EditableFogVolumeObject::DEFAULT_PROFILE)
  );
  assert_eq!(oldest.objects[0].group, EditableFogVolumeObject::UNGROUPED);
  assert!(read_fog_volumes(&fog_volume_part_text(&GUID, &[("folume_type", "2")])?).is_err());

  Ok(())
}

#[test]
fn finds_the_saved_transform_a_step_off_a_decomposed_one() -> XrfResult {
  let saved: EditableObjectTransform = EditableObjectTransform {
    position: Vector3d::new(12.5, -3.25, 40.0),
    rotation: Vector3d::new(0.0, 0.812_345, -0.25),
    scale: Vector3d::new(8.0, 4.0, 16.0),
  };
  let bits: [u32; 16] = saved.to_matrix().values.map(f32::to_bits);
  // As a box's corners measure them: the long axis a `%f` step over, an angle two under.
  let measured: EditableObjectTransform = EditableObjectTransform {
    rotation: Vector3d::new(0.0, 0.812_343, -0.25),
    scale: Vector3d::new(8.0, 4.0, 16.000_002),
    ..saved
  };

  assert_eq!(
    measured.find_saved(|transform| transform.to_matrix().values.map(f32::to_bits) == bits),
    Some(saved)
  );
  assert_eq!(
    EditableObjectTransform::from_matrix_as_saved(&saved.to_matrix())?
      .to_matrix()
      .values
      .map(f32::to_bits),
    bits
  );

  Ok(())
}

#[test]
fn reads_a_sound_environment_part_typed_and_writes_it_back_as_the_editor_saved_it() -> XrfResult {
  let text: String = sound_environment_part_text(&GUID, &[])?;
  let part: EditableObjectPart<EditableSoundEnvironmentObject> = read_sound_environments(&text)?;
  let volume: &EditableSoundEnvironmentObject = &part.objects[0];

  assert_eq!((volume.inner.as_str(), volume.outer.as_str()), ("cave", "default"));
  assert_eq!(volume.shape.usage, 0);
  assert_eq!(volume.shape.elements, [EditableShapeElement::Box(Affine3d::IDENTITY)]);
  assert!(volume.is_exported());
  // The unit box, placed by the object's own transform alone.
  assert_eq!(volume.get_box(), Some(volume.object.transform.to_matrix()));
  assert_eq!(String::from_utf8_lossy(&part.to_text_part()?.write_to_bytes()?), text);

  Ok(())
}

#[test]
fn reads_a_shapes_spheres_and_boxes_in_their_order() -> XrfResult {
  let text: String = sound_environment_part_text(
    &GUID,
    &[
      ("shapes_count", "2"),
      ("shape_type_1", "0"),
      ("shape_center_1", "1.000000, 2.000000, 3.000000"),
      ("shape_radius_1", "0.500000"),
    ],
  )?;
  let part: EditableObjectPart<EditableSoundEnvironmentObject> = read_sound_environments(&text)?;

  assert_eq!(
    part.objects[0].shape.elements[1],
    EditableShapeElement::Sphere {
      center: Vector3d::new(1.0, 2.0, 3.0),
      radius: 0.5
    }
  );
  assert_eq!(String::from_utf8_lossy(&part.to_text_part()?.write_to_bytes()?), text);

  Ok(())
}

#[test]
fn a_sound_environment_starting_with_a_sphere_has_no_box() -> XrfResult {
  let part = read_sound_environments(&sound_environment_part_text(
    &GUID,
    &[
      ("shape_type_0", "0"),
      ("shape_center_0", "0.000000, 0.000000, 0.000000"),
      ("shape_radius_0", "1.000000"),
    ],
  )?)?;

  assert_eq!(part.objects[0].get_box(), None);

  Ok(())
}

#[test]
fn refuses_a_sound_environment_the_editor_does_not_load() -> XrfResult {
  // The shape's own version left unwritten over, and an element neither a sphere nor a box.
  for field in [("version", "2"), ("shape_type_0", "3")] {
    assert!(
      read_sound_environments(&sound_environment_part_text(&GUID, &[field])?).is_err(),
      "{field:?} was read"
    );
  }

  Ok(())
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
fn refuses_a_part_of_another_tool_version_as_the_tool_does() -> XrfResult {
  let mut part: EditableTextPart = EditableTextPart::read_from_bytes(particle_part_text(&GUID, &[])?.as_bytes())?;

  part.ini.section_mut("main").set("version", "1");

  assert!(EditableObjectPart::<EditableParticleObject>::read_from_text_part(&part).is_err());

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
