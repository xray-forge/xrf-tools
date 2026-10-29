//! How `thm bump-alignment` gathers the sets a root's descriptors declare and finds their files.

use std::fs;
use std::path::{Path, PathBuf};

use xrf_chunk::XRayByteOrder;
use xrf_dds::{DdsEncoding, DdsMipChain, DdsMipmaps, ImageFormat, Quality, Rgba, RgbaImage};
use xrf_error::XrfResult;
use xrf_test_utils::GeneratedResources;
use xrf_texture::BumpAlignmentVerdict;
use xrf_thm::{ThmBumpChunk, ThmBumpMode, ThmFile};
use xrf_vfs::{XrayMountMode, XrayRoots, XraySearch};

use crate::commands::thm::bump_alignment::bump_alignment_set::{BumpAlignmentOutcome, BumpAlignmentSet};
use crate::commands::thm::bump_alignment::bump_alignment_survey::BumpAlignmentSurvey;

const SIDE: u32 = 64;

/// Tiles of random heights eight texels a side, laid out by `seed`, from nought to one.
fn tiles(seed: u32) -> Vec<f32> {
  let mut state: u32 = seed;
  let levels: Vec<f32> = (0..(SIDE / 8) * (SIDE / 8))
    .map(|_| {
      state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);

      (state >> 8) as f32 / (1u32 << 24) as f32
    })
    .collect();

  (0..SIDE * SIDE)
    .map(|index| levels[((index / SIDE / 8) * (SIDE / 8) + index % SIDE / 8) as usize])
    .collect()
}

fn write_texture(root: &Path, name: &str, image: &RgbaImage) -> XrfResult {
  DdsEncoding::new(ImageFormat::Rgba8Unorm, Quality::Fast)
    .encode(&DdsMipChain::build(image, DdsMipmaps::Disabled)?)?
    .write_to_path(&root.join("textures").join(format!("{name}.dds")))
}

/// The heights painted as a diffuse.
fn write_diffuse(root: &Path, name: &str, heights: &[f32]) -> XrfResult {
  let image: RgbaImage = RgbaImage::from_fn(SIDE, SIDE, |x, y| {
    let value: u8 = (40.0 + 180.0 * heights[(y * SIDE + x) as usize]) as u8;

    Rgba([value, value, value, u8::MAX])
  });

  write_texture(root, name, &image)
}

/// The heights as a bump, its normal reversed behind the gloss as the SDK packs it.
fn write_bump(root: &Path, name: &str, heights: &[f32]) -> XrfResult {
  let at = |x: u32, y: u32| heights[((y % SIDE) * SIDE + x % SIDE) as usize];
  let byte = |unit: f32| ((unit * 0.5 + 0.5) * 255.0).round() as u8;
  let image: RgbaImage = RgbaImage::from_fn(SIDE, SIDE, |x, y| {
    let across: f32 = (at(x + 1, y) - at(x + SIDE - 1, y)) * 4.0;
    let down: f32 = (at(x, y + 1) - at(x, y + SIDE - 1)) * 4.0;
    let length: f32 = (across * across + down * down + 1.0).sqrt();

    Rgba([128, byte(1.0 / length), byte(-down / length), byte(-across / length)])
  });

  write_texture(root, name, &image)
}

/// A texture descriptor declaring `bump`.
fn write_descriptor(root: &Path, name: &str, bump: &str) -> XrfResult {
  let mut file: ThmFile = ThmFile::new_texture();

  file.bump = Some(ThmBumpChunk {
    mode: ThmBumpMode::Use,
    name: bump.to_owned(),
    ..Default::default()
  });
  file.write_to_path::<XRayByteOrder, _>(&root.join("textures").join(format!("{name}.thm")))
}

#[test]
fn compares_each_declared_set_and_tells_its_own_bump_from_a_borrowed_one() -> XrfResult {
  let root: PathBuf = GeneratedResources::create_dir("thm/bump_alignment/survey")?;

  fs::create_dir_all(root.join("textures"))?;

  write_diffuse(&root, "stone", &tiles(7))?;
  write_bump(&root, "stone_bump", &tiles(7))?;
  write_descriptor(&root, "stone", "stone_bump")?;
  // Another picture over the stone's relief.
  write_diffuse(&root, "patch", &tiles(91))?;
  write_descriptor(&root, "patch", "stone_bump")?;
  // A descriptor left behind by a texture since removed.
  write_descriptor(&root, "removed", "stone_bump")?;
  // A descriptor declaring no bump is no set.
  ThmFile::new_texture().write_to_path::<XRayByteOrder, _>(&root.join("textures").join("plain.thm"))?;

  let search: XraySearch = XrayRoots::one(root, XrayMountMode::Directory)
    .to_search_plan()?
    .open()?;
  let survey: BumpAlignmentSurvey = BumpAlignmentSurvey::run(&search, &search);
  let find = |name: &str| {
    survey
      .sets
      .iter()
      .find(|it| it.descriptor == format!("textures\\{name}.thm"))
      .expect("a set")
  };
  let verdict_of = |set: &BumpAlignmentSet| set.get_alignment().map(|it| it.verdict);

  assert_eq!((survey.descriptors, survey.sets.len()), (4, 3));

  let (stone, patch, removed): (&BumpAlignmentSet, &BumpAlignmentSet, &BumpAlignmentSet) =
    (find("stone"), find("patch"), find("removed"));

  assert_eq!(stone.bump_file.as_deref(), Some("textures\\stone_bump.dds"));
  assert_eq!(
    (stone.is_borrowed, verdict_of(stone)),
    (false, Some(BumpAlignmentVerdict::Aligned))
  );
  assert_eq!(
    (patch.is_borrowed, verdict_of(patch)),
    (true, Some(BumpAlignmentVerdict::Mismatched))
  );
  assert!(matches!(removed.outcome, BumpAlignmentOutcome::MissingDiffuse));

  Ok(())
}
