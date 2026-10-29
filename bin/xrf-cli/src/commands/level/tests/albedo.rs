//! How `level albedo` gathers a level's base textures and finds them as the renderer does.

use std::fs;
use std::path::{Path, PathBuf};

use xrf_dds::{DdsEncoding, DdsMipChain, DdsMipmaps, ImageFormat, Quality, Rgba, RgbaImage};
use xrf_error::XrfResult;
use xrf_level::{LevelFile, LevelHeaderChunk, LevelShaderEntry, LevelShaderReference, LevelShadersChunk};
use xrf_material::XrayTextureScope;
use xrf_test_utils::GeneratedResources;
use xrf_texture::AlbedoVerdict;
use xrf_vfs::{XrayLogicalPath, XrayMountMode, XrayRoots, XraySearch};

use crate::commands::level::albedo::albedo_texture::{AlbedoOutcome, AlbedoTexture};
use crate::commands::level::albedo::level_albedo_survey::LevelAlbedoSurvey;

/// A flat grey texture of `value` written to `path` under `root`.
fn write_grey(root: &Path, path: &str, value: u8) -> XrfResult {
  let target: PathBuf = root.join(path);
  let image: RgbaImage = RgbaImage::from_pixel(8, 8, Rgba([value, value, value, u8::MAX]));

  fs::create_dir_all(target.parent().expect("a texture directory"))?;

  DdsEncoding::new(ImageFormat::Rgba8Unorm, Quality::Fast)
    .encode(&DdsMipChain::build(&image, DdsMipmaps::Disabled)?)?
    .write_to_path(&target)
}

fn reference(shader: &str, textures: &[&str]) -> LevelShaderEntry {
  LevelShaderEntry::Reference(LevelShaderReference {
    shader: shader.to_owned(),
    textures: textures.iter().map(|it| (*it).to_owned()).collect(),
  })
}

fn level(entries: Vec<LevelShaderEntry>) -> LevelFile {
  LevelFile {
    header: LevelHeaderChunk {
      xrlc_version: 18,
      xrlc_quality: 0,
    },
    shaders: Some(LevelShadersChunk { entries }),
    sectors: None,
    portals: None,
    lights: None,
    glows: None,
  }
}

#[test]
fn measures_each_base_texture_once_as_the_level_finds_it() -> XrfResult {
  let root: PathBuf = GeneratedResources::create_dir("level/albedo/survey")?;

  write_grey(&root, "textures/grnd/grey.dds", 128)?;
  write_grey(&root, "textures/shadowed.dds", 200)?;
  // The level's own copy, which the renderer binds over the shared one.
  write_grey(&root, "levels/test/shadowed.dds", 20)?;

  let search: XraySearch = XrayRoots::one(root, XrayMountMode::Directory)
    .to_search_plan()?
    .open()?;
  let scope: XrayTextureScope = XrayTextureScope::of_level(XrayLogicalPath::new("levels\\test")?);
  let survey: LevelAlbedoSurvey = LevelAlbedoSurvey::run(
    &search,
    scope,
    &level(vec![
      reference("def_shaders\\def_vertex", &["grnd\\grey"]),
      reference("def_shaders\\def_lmap", &["shadowed", "lmap#1_1"]),
      reference("def_shaders\\def_trans", &["grnd\\GREY"]),
      reference("def_shaders\\def_vertex", &["absent"]),
      LevelShaderEntry::Empty,
    ]),
  );

  assert_eq!((survey.surfaces, survey.unlit_surfaces), (4, 0));

  let references: Vec<&str> = survey.textures.iter().map(|it| it.reference.as_str()).collect();

  assert_eq!(
    references,
    ["absent", "grnd\\grey", "shadowed"],
    "one entry a texture, whatever its case"
  );

  let [absent, grey, shadowed]: [&AlbedoTexture; 3] = [0, 1, 2].map(|index| &survey.textures[index]);

  assert!(matches!(absent.outcome, AlbedoOutcome::Missing));
  assert_eq!(absent.file, None);
  assert_eq!(grey.shaders.len(), 2, "both shaders drawing it are named");
  assert_eq!(grey.get_albedo().map(|it| it.verdict), Some(AlbedoVerdict::Within));
  assert_eq!(shadowed.file.as_deref(), Some("levels\\test\\shadowed.dds"));
  assert_eq!(
    shadowed.get_albedo().map(|it| it.verdict),
    Some(AlbedoVerdict::Dark),
    "measured from the level's copy, not the shared one"
  );

  Ok(())
}
