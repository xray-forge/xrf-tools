use std::f32::consts::FRAC_PI_4;

use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_editable_level::{
  EditableCustomObject, EditableLevelValue, EditableObjectPart, EditableObjectTransform, EditableShape,
  EditableShapeElement, EditableSoundEnvironmentObject, EditableTextPart,
};
use xrf_error::XrfResult;
use xrf_level::LevelSndEnvFile;
use xrf_math::{Affine3d, Vector3d};

use crate::level::tests::fixtures::GUID;
use crate::{LevelDecompileContext, LevelSoundEnvironmentConversion};

/// A box sheared by its own matrix, which no object transform of the unit box gives.
const SHEARED: Affine3d = Affine3d {
  x_axis: Vector3d::new(2.0, 0.0, 0.5),
  y_axis: Vector3d::new(0.0, 3.0, 0.0),
  z_axis: Vector3d::new(0.25, 0.0, 4.0),
  translation: Vector3d::new(10.0, 1.0, -5.0),
};

fn transform(position: [f32; 3], rotation: [f32; 3], scale: [f32; 3]) -> EditableObjectTransform {
  EditableObjectTransform {
    position: Vector3d::from(position),
    rotation: Vector3d::from(rotation),
    scale: Vector3d::from(scale),
  }
}

fn volume(
  inner: &str,
  outer: &str,
  transform: EditableObjectTransform,
  matrix: Affine3d,
) -> EditableSoundEnvironmentObject {
  EditableSoundEnvironmentObject {
    object: EditableCustomObject::new("authored", transform),
    shape: EditableShape {
      usage: 0,
      elements: vec![EditableShapeElement::Box(matrix)],
    },
    inner: inner.to_owned(),
    outer: outer.to_owned(),
  }
}

fn part(volumes: Vec<EditableSoundEnvironmentObject>) -> EditableObjectPart<EditableSoundEnvironmentObject> {
  EditableObjectPart::new(GUID, volumes)
}

fn export(part: &EditableObjectPart<EditableSoundEnvironmentObject>) -> XrfResult<LevelSndEnvFile> {
  Ok(LevelSoundEnvironmentConversion::export(part)?.expect("a file"))
}

fn bytes_of(file: &LevelSndEnvFile) -> XrfResult<Vec<u8>> {
  let mut writer: ChunkWriter = ChunkWriter::new();

  file.write::<XRayByteOrder>(&mut writer)?;

  Ok(writer.buffer)
}

fn decompile(file: &LevelSndEnvFile) -> XrfResult<EditableObjectPart<EditableSoundEnvironmentObject>> {
  LevelSoundEnvironmentConversion::decompile(file, &mut LevelDecompileContext::new(GUID))
}

#[test]
fn environments_come_back_to_the_same_file_through_the_parts_text() -> XrfResult {
  // An eighth of a turn about `y`: the box's long axis measures `16.000002` off the corners, a `%f` step over.
  let eighth_turn: f32 = EditableLevelValue::round_as_saved(FRAC_PI_4);
  let file: LevelSndEnvFile = export(&part(vec![
    volume(
      "cave",
      "default",
      transform([12.5, -3.25, 40.0], [0.0, eighth_turn, 0.0], [8.0, 4.0, 16.0]),
      Affine3d::IDENTITY,
    ),
    volume(
      "hall",
      "default",
      transform([-30.0, 2.0, 7.5], [0.1, -1.2, 0.05], [3.0, 3.0, 3.0]),
      Affine3d::IDENTITY,
    ),
    volume("cave", "hall", EditableObjectTransform::default(), SHEARED),
  ]))?;
  let decompiled = decompile(&file)?;
  let text: Vec<u8> = decompiled.to_text_part()?.write_to_bytes()?;
  let read: EditableObjectPart<EditableSoundEnvironmentObject> =
    EditableObjectPart::read_from_text_part(&EditableTextPart::read_from_bytes(&text)?)?;
  let names: Vec<&str> = read.objects.iter().map(|it| it.object.name.as_str()).collect();

  assert_eq!(
    file.environments,
    ["cave", "default", "hall"],
    "inner first, as volumes first name them"
  );
  assert_eq!(names, ["sound_env_0000", "sound_env_0001", "sound_env_0002"]);
  assert_eq!(
    read.objects[0].shape,
    EditableShape::new_unit_box(),
    "the editor's own volume"
  );
  assert_eq!(read.objects[2].shape.elements, [EditableShapeElement::Box(SHEARED)]);
  assert_eq!(read.objects[2].object.transform, EditableObjectTransform::default());
  assert_eq!(
    (read.objects[2].inner.as_str(), read.objects[2].outer.as_str()),
    ("cave", "hall")
  );
  assert_eq!(bytes_of(&export(&read)?)?, bytes_of(&file)?);

  Ok(())
}

#[test]
fn a_volume_the_export_skips_widens_only_the_bounds() -> XrfResult {
  let exported = volume(
    "cave",
    "default",
    transform([0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [2.0, 2.0, 2.0]),
    Affine3d::IDENTITY,
  );
  let skipped = volume(
    "cave",
    "cave",
    transform([100.0, 0.0, 0.0], [0.0, 0.0, 0.0], [2.0, 2.0, 2.0]),
    Affine3d::IDENTITY,
  );
  let alone: LevelSndEnvFile = export(&part(vec![exported.clone()]))?;
  let both: LevelSndEnvFile = export(&part(vec![exported, skipped]))?;

  assert_eq!((both.vertices.len(), both.faces.len()), (8, 12));
  assert_eq!(both.bounds.min, Vector3d::new(-1.0, -1.0, -1.0));
  assert_eq!(both.bounds.max, Vector3d::new(101.0, 1.0, 1.0));
  // A decompile cannot know the skipped volume: it gives back the one exported, and the bounds it held.
  assert_eq!(decompile(&both)?.objects.len(), 1);
  assert_eq!(export(&decompile(&both)?)?.bounds, alone.bounds);

  Ok(())
}

#[test]
fn a_part_of_no_exported_volume_exports_no_file() -> XrfResult {
  let skipped = volume("", "default", EditableObjectTransform::default(), Affine3d::IDENTITY);

  assert!(LevelSoundEnvironmentConversion::export(&part(vec![skipped]))?.is_none());
  assert!(LevelSoundEnvironmentConversion::export(&part(Vec::new()))?.is_none());

  Ok(())
}

#[test]
fn merges_corners_within_eps_as_the_editors_collector_does() -> XrfResult {
  // Two unit boxes side by side share a face's four corners, which the export writes once.
  let file: LevelSndEnvFile = export(&part(vec![
    volume(
      "cave",
      "default",
      EditableObjectTransform::default(),
      Affine3d::IDENTITY,
    ),
    volume(
      "cave",
      "default",
      transform([1.0, 0.0, 0.0], [0.0, 0.0, 0.0], [1.0, 1.0, 1.0]),
      Affine3d::IDENTITY,
    ),
  ]))?;

  assert_eq!((file.vertices.len(), file.faces.len()), (12, 24));
  assert!(decompile(&file).is_err(), "a merged mesh is not whole boxes");

  Ok(())
}

#[test]
fn refuses_what_the_editor_asserts_on_and_bounds_a_skipped_sphere() -> XrfResult {
  let boxed = volume(
    "cave",
    "default",
    EditableObjectTransform::default(),
    Affine3d::IDENTITY,
  );
  let mut sphere = boxed.clone();
  let mut empty = boxed.clone();

  sphere.shape.elements = vec![EditableShapeElement::Sphere {
    center: Vector3d::new(5.0, 0.0, 0.0),
    radius: 1.0,
  }];
  empty.shape.elements.clear();

  // `get_box` asserts only for a volume it exports, `GetBox` for every one.
  assert!(LevelSoundEnvironmentConversion::export(&part(vec![sphere.clone()])).is_err());
  assert!(LevelSoundEnvironmentConversion::export(&part(vec![boxed.clone(), empty])).is_err());

  sphere.outer = sphere.inner.clone();

  assert_eq!(
    export(&part(vec![boxed, sphere]))?.bounds.max,
    Vector3d::new(6.0, 1.0, 1.0)
  );

  Ok(())
}
