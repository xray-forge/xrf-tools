use std::collections::BTreeMap;

use xrf_editable_level::EditableObjectTransform;
use xrf_editable_object::EditableObjectFile;
use xrf_error::XrfResult;
use xrf_gamemtl::GameMtlFile;
use xrf_level::{LevelHomFile, LevelSomFile};
use xrf_math::Vector3d;

use crate::level::level_geometry_conversion::LevelDecompiledObject;
use crate::level::level_visual_geometry::LevelVisualGeometry;
use crate::object::decompile::geometry::source::surface_dress::SurfaceDress;
use crate::object::decompile::object_decompiler::ObjectDecompiler;

/// A level's occluders back into the objects the Build exports them from (`SceneBuilder::BuildHOMModel` and
/// `BuildSOMModel`, `BuilderOGF.cpp:17`, `:94`), each polygon in the level's space, two-sided where the file says so.
///
/// - `level.hom` takes every face of a surface whose game material is `materials\occ` (or of an object flagged
///   `eoHOM`); the decompile follows GSC's own `labx8`, an ordinary object of such a surface, `<level>_hom`.
/// - `level.som` takes every face of an object flagged `eoSoundOccluder`, with its surface's game material's sound
///   occlusion factor; the decompile gives `<level>_som` that flag, and each surface the game material of the library
///   whose factor is the polygon's.
pub struct LevelOccluderConversion;

/// What a sound occluder's decompile could not give back exactly.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LevelSoundOccluderCensus {
  pub polygons: usize,
  /// Occlusion factors no game material of the library has, whose polygons take `default`.
  pub unmatched_factors: Vec<f32>,
}

impl LevelOccluderConversion {
  /// The game material whose faces the Build exports to `level.hom` (`BuilderOGF.cpp:64`), which it leaves out of the
  /// level's geometry (`BuilderRemote.cpp:539`).
  pub const OCCLUDER_GAME_MATERIAL: &'static str = "materials\\occ";

  /// How GSC dressed `labx8`'s occluder object, which both objects take beside their game materials.
  pub const OCCLUDER_TEXTURE: &'static str = "prop\\prop_fake_occ";
  pub const OCCLUDER_SHADER: &'static str = "default";

  /// `CEditableObject::eoSoundOccluder`.
  pub const SOUND_OCCLUDER_FLAG: u32 = 1 << 5;

  /// `level.hom` as `<level>_hom`: one surface of `materials\occ`, two-sided where its polygons are, at the origin.
  ///
  /// # Errors
  ///
  /// Returns an error when the decompile refuses the polygons.
  pub fn decompile_hom(name: &str, file: &LevelHomFile) -> XrfResult<Option<LevelDecompiledObject>> {
    let polygons: Vec<([Vector3d; 3], bool)> = file
      .polygons
      .iter()
      .map(|polygon| (polygon.vertices, polygon.flags != 0))
      .collect();

    Self::decompile(
      format!("levels\\{name}\\{name}_hom"),
      LevelHomFile::FILE_NAME,
      [(String::from(Self::OCCLUDER_GAME_MATERIAL), polygons)].into(),
      0,
    )
  }

  /// `level.som` as `<level>_som`, flagged a sound occluder: a surface per game material whose sound occlusion factor
  /// is a polygon's, found in `game_materials`, two-sided where its polygons are.
  ///
  /// # Errors
  ///
  /// Returns an error when the decompile refuses the polygons.
  pub fn decompile_som(
    name: &str,
    file: &LevelSomFile,
    game_materials: Option<&GameMtlFile>,
  ) -> XrfResult<(Option<LevelDecompiledObject>, LevelSoundOccluderCensus)> {
    let mut census: LevelSoundOccluderCensus = LevelSoundOccluderCensus {
      polygons: file.polygons.len(),
      ..LevelSoundOccluderCensus::default()
    };
    let mut by_material: BTreeMap<String, Vec<([Vector3d; 3], bool)>> = BTreeMap::new();

    for polygon in &file.polygons {
      let material: String = match Self::find_material_by_factor(game_materials, polygon.occlusion) {
        Some(material) => material,
        None => {
          if !census
            .unmatched_factors
            .iter()
            .any(|factor| factor.to_bits() == polygon.occlusion.to_bits())
          {
            census.unmatched_factors.push(polygon.occlusion);
          }

          String::from(SurfaceDress::DEFAULT_SETTING)
        }
      };

      by_material
        .entry(material)
        .or_default()
        .push((polygon.vertices, polygon.is_two_sided != 0));
    }

    Ok((
      Self::decompile(
        format!("levels\\{name}\\{name}_som"),
        LevelSomFile::FILE_NAME,
        by_material,
        Self::SOUND_OCCLUDER_FLAG,
      )?,
      census,
    ))
  }

  /// The game material of the lowest number whose sound occlusion factor is `factor`, bit for bit.
  fn find_material_by_factor(game_materials: Option<&GameMtlFile>, factor: f32) -> Option<String> {
    game_materials?
      .materials
      .iter()
      .filter(|material| material.sound_occlusion_factor.to_bits() == factor.to_bits())
      .min_by_key(|material| material.id)
      .map(|material| material.name.clone())
  }

  /// An object at the origin of `flags` holding the polygons of each game material, a two-sided polygon given with its
  /// mirror so the decompile folds the two into one two-sided face.
  fn decompile(
    reference: String,
    source: &'static str,
    by_material: BTreeMap<String, Vec<([Vector3d; 3], bool)>>,
    flags: u32,
  ) -> XrfResult<Option<LevelDecompiledObject>> {
    let mut geometry: Vec<LevelVisualGeometry> = Vec::new();

    for (material, polygons) in by_material {
      let dress: SurfaceDress = SurfaceDress {
        compiler_shader: String::from(Self::OCCLUDER_SHADER),
        game_material: material,
      };
      let triangles: Vec<[Vector3d; 3]> = polygons
        .iter()
        .flat_map(|(points, is_two_sided)| {
          std::iter::once(*points).chain(is_two_sided.then_some([points[0], points[2], points[1]]))
        })
        .collect();

      geometry.extend(LevelVisualGeometry::of_triangles(
        Self::OCCLUDER_TEXTURE,
        Self::OCCLUDER_SHADER,
        &dress,
        &triangles,
      ));
    }

    if geometry.is_empty() {
      return Ok(None);
    }

    let mut decompile = ObjectDecompiler::decompile_level_geometry(&geometry)?;
    let object: &mut EditableObjectFile = &mut decompile.object;

    object.flags |= flags;

    Ok(Some(LevelDecompiledObject {
      reference,
      transform: EditableObjectTransform::default(),
      source: String::from(source),
      decompile,
    }))
  }
}
