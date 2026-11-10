use xrf_editable_object::EditableObjectFile;
use xrf_error::XrfResult;
use xrf_gamemtl::{GameMtlFile, GameMtlMaterial};
use xrf_level::{LevelHomFile, LevelHomPolygon, LevelSomFile, LevelSomPolygon};
use xrf_math::Vector3d;

use crate::{LevelDecompiledObject, LevelOccluderConversion};

/// A triangle standing up at `x`.
fn wall(x: f32) -> [Vector3d; 3] {
  [
    Vector3d::new(x, 0.0, 0.0),
    Vector3d::new(x, 3.0, 0.0),
    Vector3d::new(x + 2.0, 0.0, 0.0),
  ]
}

/// Each surface's name, game material and whether it is two-sided, with the faces it holds.
fn surfaces_of(object: &EditableObjectFile) -> Vec<(String, bool, usize)> {
  let mesh = &object.meshes.as_deref().unwrap_or_default()[0];

  object
    .surfaces
    .as_deref()
    .unwrap_or_default()
    .iter()
    .map(|surface| {
      let faces: usize = mesh
        .surface_faces
        .iter()
        .find(|faces| faces.surface == surface.name)
        .map_or(0, |faces| faces.faces.len());

      (surface.game_material.clone(), surface.is_two_sided(), faces)
    })
    .collect()
}

#[test]
fn takes_the_occlusion_mesh_back_as_a_surface_of_the_occluder_material() -> XrfResult {
  let hom: LevelHomFile = LevelHomFile {
    version: LevelHomFile::CURRENT_VERSION,
    polygons: vec![
      LevelHomPolygon {
        vertices: wall(0.0),
        flags: 1,
      },
      LevelHomPolygon {
        vertices: wall(10.0),
        flags: 0,
      },
    ],
  };
  let object: LevelDecompiledObject =
    LevelOccluderConversion::decompile_hom("test", &hom)?.expect("an occlusion mesh of polygons is an object");
  let occluder = String::from(LevelOccluderConversion::OCCLUDER_GAME_MATERIAL);

  assert_eq!(object.reference, "levels\\test\\test_hom");
  assert_eq!(object.source, "level.hom");
  assert_eq!(
    object.decompile.object.flags, 0,
    "an ordinary object, as GSC's labx8 keeps its occluders"
  );
  assert_eq!(
    surfaces_of(&object.decompile.object),
    [(occluder.clone(), false, 1), (occluder, true, 1)],
    "the two-sided polygon folds with its mirror into one two-sided face"
  );

  Ok(())
}

#[test]
fn takes_sound_occluders_back_with_the_material_of_their_occlusion_factor() -> XrfResult {
  let polygon = |x: f32, occlusion: f32| LevelSomPolygon {
    vertices: wall(x),
    is_two_sided: 0,
    occlusion,
  };
  let som: LevelSomFile = LevelSomFile {
    version: 0,
    polygons: vec![polygon(0.0, 0.5), polygon(10.0, 0.9)],
  };
  let mut brick: GameMtlMaterial = GameMtlMaterial::new(4, "materials\\brick");
  let mut stone: GameMtlMaterial = GameMtlMaterial::new(2, "materials\\stone");

  brick.sound_occlusion_factor = 0.5;
  stone.sound_occlusion_factor = 0.5;

  let library: GameMtlFile = GameMtlFile::new(vec![brick, stone], Vec::new());
  let (object, census) = LevelOccluderConversion::decompile_som("test", &som, Some(&library))?;
  let object: LevelDecompiledObject = object.expect("a sound occlusion mesh of polygons is an object");

  assert_eq!(object.reference, "levels\\test\\test_som");
  assert_eq!(object.source, "level.som");
  assert_eq!(
    object.decompile.object.flags,
    LevelOccluderConversion::SOUND_OCCLUDER_FLAG
  );
  assert_eq!(
    surfaces_of(&object.decompile.object),
    [
      (String::from("default"), false, 1),
      (String::from("materials\\stone"), false, 1)
    ],
    "the lowest number of the materials sharing a factor; default for a factor none has"
  );
  assert_eq!(census.unmatched_factors, [0.9]);

  Ok(())
}
