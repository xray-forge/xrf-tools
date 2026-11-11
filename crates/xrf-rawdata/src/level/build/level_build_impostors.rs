use std::path::{Path, PathBuf};

use xrf_dds::{DdsFile, RgbaImage};
use xrf_editable_object::EditableObjectFile;
use xrf_error::{XrfError, XrfResult};
use xrf_level::{BuildPrjLod, BuildPrjLodFace};
use xrf_math::{Aabb, Vector2d};
use xrf_utils::format_path;

use crate::level::build::level_build_atlas::LevelBuildAtlas;
use crate::level::build::level_build_tables::LevelBuildTables;
use crate::level::level_lod_strip::LevelLodStrip;
use crate::level::scene::level_scene_decompile::LevelSceneTextureFile;

/// The impostors a build draws its multi-use models as from afar (`SceneBuilder::BuildObjectLOD`,
/// `BuilderLODs.cpp:54`): for each model flagged `eoUsingLOD`, eight quads standing through its box in its own space,
/// drawn from its strips `lod\lod_<model>` and `_nm`, which the Build merges into the level's impostor atlas
/// (`BuilderRemote.cpp:1516`).
#[derive(Default)]
pub(crate) struct LevelBuildImpostors {
  lods: Vec<BuildPrjLod>,
  /// Each impostor's strip and its normals, as loaded.
  strips: Vec<[RgbaImage; 2]>,
}

/// The impostor atlas a build writes beside `build.prj`, as the level ships it: `level_lods.dds` and
/// `level_lods_nm.dds`, DXT5 with mips.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LevelBuildImpostorAtlas {
  pub color: Vec<u8>,
  pub normal: Vec<u8>,
}

impl LevelBuildImpostors {
  /// The texture and render shader of every impostor's material (`LEVEL_LODS_TEX_NAME`, `LOD_SHADER_NAME`).
  pub(crate) const TEXTURE: &'static str = "level_lods";
  pub(crate) const SHADER: &'static str = "details\\lod";

  /// Adds the impostor of `object`, the model of `reference`, in `sector`, its strips read under `textures`; its index,
  /// which the model's `lod` names. Its material goes in first, before the model's own, as the Build adds it.
  ///
  /// # Errors
  ///
  /// Returns an error, as the Build stops, when a strip is missing, does not read or is not one of eight 64 by 64
  /// frames, or the object holds no mesh to stand the quads in.
  pub(crate) fn add(
    &mut self,
    reference: &str,
    object: &EditableObjectFile,
    sector: u16,
    tables: &mut LevelBuildTables,
    textures: &Path,
  ) -> XrfResult<u16> {
    let material: u16 = tables.get_material(Self::SHADER, None, Self::TEXTURE, sector)?;
    let bounds: Aabb = object
      .meshes
      .iter()
      .flatten()
      .map(|mesh| mesh.bounds)
      .reduce(|mut held, bounds| {
        held.include_box(&bounds);
        held
      })
      .ok_or_else(|| {
        XrfError::new_invalid_error(format!(
          "Multi-use object '{reference}' has an impostor but no mesh to stand it in"
        ))
      })?;
    let name: String = LevelLodStrip::get_texture_name(reference);
    let color: RgbaImage = Self::read_strip(textures, &name)?;
    let normal: RgbaImage = Self::read_strip(textures, &(name + LevelLodStrip::NORMAL_SUFFIX))?;
    let faces: [BuildPrjLodFace; BuildPrjLod::FACES] = std::array::from_fn(|frame| BuildPrjLodFace {
      vertices: LevelLodStrip::get_frame_corners(frame, &bounds),
      texture_coordinates: LevelLodStrip::get_frame_coordinates(frame),
    });

    self.lods.push(BuildPrjLod {
      faces,
      material: u32::from(material),
    });
    self.strips.push([color, normal]);

    u16::try_from(self.lods.len() - 1)
      .ok()
      .filter(|index| *index != u16::MAX)
      .ok_or_else(|| XrfError::new_invalid_error("A build holds more impostors than its u16 indices number"))
  }

  /// The project's `b_lod`s with their coordinates moved into the atlas of their strips, and the atlas as the Build
  /// saves it; `None` where there is no impostor.
  ///
  /// # Errors
  ///
  /// Returns an error, as the Build stops, when the strips do not fit an atlas 2048 by 2048, or the atlas does not
  /// encode.
  pub(crate) fn into_export(mut self) -> XrfResult<Option<(Vec<BuildPrjLod>, LevelBuildImpostorAtlas)>> {
    if self.lods.is_empty() {
      return Ok(None);
    }

    let atlas: LevelBuildAtlas = LevelBuildAtlas::merge(
      &self.strips.iter().map(<[RgbaImage; 2]>::as_slice).collect::<Vec<_>>(),
      &LevelBuildAtlas::LOD_SIZES,
    )
    .ok_or_else(|| {
      let (width, height) = LevelBuildAtlas::LOD_SIZES.largest;

      XrfError::new_invalid_error(format!(
        "The {} impostors' strips do not fit an atlas {width} by {height}",
        self.strips.len()
      ))
    })?;

    for (lod, place) in self.lods.iter_mut().zip(&atlas.places) {
      for face in &mut lod.faces {
        for coordinate in &mut face.texture_coordinates {
          let (u, v) = place.remap(coordinate.x, coordinate.y);

          *coordinate = Vector2d::new(u, v);
        }
      }
    }

    Ok(Some((
      self.lods,
      LevelBuildImpostorAtlas {
        color: LevelLodStrip::encode(&atlas.layers[0])?,
        normal: LevelLodStrip::encode(&atlas.layers[1])?,
      },
    )))
  }

  /// The strip `name` under `textures`, a `.dds` of eight frames side by side, where the decompile writes it.
  fn read_strip(textures: &Path, name: &str) -> XrfResult<RgbaImage> {
    let path: PathBuf = LevelSceneTextureFile::get_named_path(textures, name);
    let image: RgbaImage = DdsFile::read_from_path(&path)
      .and_then(|file| file.decode_rgba(0))
      .map_err(|error| {
        XrfError::new_read_error(format!(
          "Impostor strip '{name}' was not read from {}: {error}",
          format_path(&path)
        ))
      })?;

    if image.dimensions() != (LevelLodStrip::WIDTH, LevelLodStrip::HEIGHT) {
      return Err(XrfError::new_invalid_error(format!(
        "Impostor strip '{name}' is {:?}, not {} by {}",
        image.dimensions(),
        LevelLodStrip::WIDTH,
        LevelLodStrip::HEIGHT
      )));
    }

    Ok(image)
  }
}
