//! `level.ltx` read back into a scene's map settings and written again as the Build writes it.

use xrf_chunk::XRayByteOrder;
use xrf_editable_level::{
  EditableCustomObject, EditableLevelFile, EditableLevelOptions, EditableObjectPart, EditableObjectTransform,
  EditableSectorObject, EditableShape, EditableShapeObject,
};
use xrf_error::XrfResult;
use xrf_test_utils::GeneratedResources;

use crate::level::level_decompile_context::LevelDecompileContext;
use crate::level::tests::fixtures::GUID;
use crate::{
  LevelDecompileOptions, LevelMapConversion, LevelMapSettings, LevelSceneDecompile, LevelSceneDecompiler,
  LevelSceneInference,
};

/// `labx8`'s `level.ltx` as shipped, saved again after the Build: spaced keys, blank lines; with two game types, an
/// extra section and a build options' line ended by a zero byte.
const SHIPPED: &[u8] =
  b"[map_usage]\r\nver = 1.0\r\ndeathmatch\r\ncapturetheartefact\r\nfreemp\r\n;\r\n\r\n[level_map]\r\n\
bound_rect = -122.440560,44.614076,-40.440560,126.614072\r\ntexture = map\\map_labx8\r\n;\r\n\
[deathmatch_game_anomaly_sets]\r\nset1 = zone_mine\0;\r\n\r\n[sub_level_map]\r\n0 = 255\r\n1 = 0\r\n2 = 3\r\n";

/// The shipped rectangle as the decompile reads it.
fn shipped_bounds() -> [f32; 4] {
  ["-122.440560", "44.614076", "-40.440560", "126.614072"].map(|value| value.parse::<f32>().unwrap_or_default())
}

#[test]
fn reads_the_maps_settings_as_the_build_wrote_them_and_a_later_save_left_them() -> XrfResult {
  let settings: LevelMapSettings = LevelMapConversion::decompile(SHIPPED)?;

  assert_eq!(settings.map_version.as_deref(), Some("1.0"));
  assert_eq!(settings.game_types, (1 << 1) | (1 << 4));
  assert_eq!(settings.unknown_game_types, ["freemp"]);
  assert_eq!(settings.bounds, Some(shipped_bounds()));
  assert_eq!(settings.texture.as_deref(), Some("map\\map_labx8"));
  assert_eq!(
    settings.build_options,
    "[deathmatch_game_anomaly_sets]\r\nset1 = zone_mine"
  );
  assert_eq!(
    settings.sector_maps.into_iter().collect::<Vec<_>>(),
    [(0, 255), (1, 0), (2, 3)]
  );
  assert_eq!(
    LevelMapConversion::decompile(b"[map_usage]\r\nver=1.0\r\n")?.game_types,
    EditableLevelOptions::SINGLE_PLAYER,
    "a map naming no game type is single player's"
  );
  assert!(LevelMapConversion::decompile(b"[level_map]\r\nbound_rect = 1,2,3\r\n").is_err());
  assert!(LevelMapConversion::decompile(b"[sub_level_map]\r\nzero = 1\r\n").is_err());

  Ok(())
}

#[test]
fn gives_the_maps_rectangle_back_as_a_level_bound_shape_whose_box_writes_it_again() -> XrfResult {
  let shape: EditableShapeObject =
    LevelMapConversion::decompile_bound(&shipped_bounds(), &mut LevelDecompileContext::new(GUID));
  let placed = shape
    .shape
    .get_bounds()
    .expect("a box element")
    .transform(&shape.object.transform.to_matrix());

  assert_eq!(shape.shape.usage, EditableShape::LEVEL_BOUND_USAGE);
  assert_eq!(shape.object.name, "level_bound_0000");
  assert_eq!(
    [placed.min.x, placed.min.z, placed.max.x, placed.max.z],
    shipped_bounds(),
    "the box ends exactly on the rectangle read"
  );

  let sector = |map_index: u8| EditableSectorObject {
    object: EditableCustomObject::new(String::from("sector"), EditableObjectTransform::default()),
    color: [255; 4],
    is_default: false,
    items: Vec::new(),
    map_index,
  };
  let written: String = String::from_utf8_lossy(&LevelMapConversion::export(
    &EditableLevelOptions::new("labx8", EditableLevelOptions::SINGLE_PLAYER),
    &placed,
    &[sector(255), sector(0)],
  ))
  .into_owned();

  // GSC's file holds decimals no float prints (`126.614072` lies between two), so the floats they read as come back.
  assert_eq!(
    written,
    "[map_usage]\r\nver=1.0\r\n;\r\n[level_map]\r\n\
     bound_rect = -122.440559,44.614075,-40.440559,126.614075\r\ntexture = map\\map_labx8\r\n;\r\n;\r\n\
     [sub_level_map]\r\n0 = 255\r\n1 = 0\r\n"
  );

  Ok(())
}

#[test]
fn decompiles_a_levels_map_settings_into_its_options_and_a_level_bound_shape() -> XrfResult {
  let directory = GeneratedResources::create_dir(&GeneratedResources::sample_name(file!(), "map"))?;

  std::fs::write(directory.join(LevelMapConversion::FILE_NAME), SHIPPED)?;

  let decompiled: LevelSceneDecompile = LevelSceneDecompiler::decompile::<XRayByteOrder>(
    &directory,
    &LevelDecompileOptions::new("labx8", "test", "1.0.0"),
  )?;
  let options: EditableLevelOptions = decompiled.scene.level.get_level_options()?;
  let shapes: EditableObjectPart<EditableShapeObject> =
    EditableObjectPart::read_from_scene_tool(decompiled.scene.get_tool("shape").expect("a shape part"))?;

  assert_eq!(options.game_type, (1 << 1) | (1 << 4));
  assert_eq!(shapes.objects.len(), 1);
  assert!(decompiled.provenance.inferred.contains(&LevelSceneInference::MapBound));
  // The anomaly sets travel as the build options' text, across lines in the saved scene as the editor saves it.
  assert_eq!(
    EditableLevelFile::read_from_bytes(&decompiled.scene.level.write_to_bytes()?)?
      .get_level_options()?
      .build_options,
    "[deathmatch_game_anomaly_sets]\r\nset1 = zone_mine"
  );
  assert!(
    decompiled
      .provenance
      .inferred
      .contains(&LevelSceneInference::BuildOptions)
  );
  assert!(
    !decompiled.provenance.inferred.contains(&LevelSceneInference::GameTypes),
    "the map names the game types"
  );
  assert_eq!(
    decompiled
      .provenance
      .refused
      .iter()
      .map(|refusal| refusal.object.as_str())
      .collect::<Vec<_>>(),
    ["[map_usage] freemp"]
  );

  Ok(())
}
