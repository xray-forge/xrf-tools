use xrf_chunk::XRayByteOrder;
use xrf_error::XrfResult;
use xrf_ogf::{OgfChildrenLinkChunk, OgfGeometryContainerChunk, OgfHeaderChunk};

use crate::level::level_file::LevelFile;
use crate::level::level_sector_composition::LevelSectorComposition;
use crate::level::level_visuals_chunk::LevelVisualsChunk;
use crate::tests::fixtures::{chunk, floats};

/// An OGF header of the given model type, which is all a composition walk reads of one.
fn new_header(model_type: u8) -> XrfResult<Vec<u8>> {
  let mut bytes: Vec<u8> = vec![4, model_type, 0, 0];

  bytes.extend(floats(&[0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 1.0]));

  chunk(OgfHeaderChunk::CHUNK_ID, &bytes)
}

/// A geometry container, which is what makes a visual drawable.
fn new_container() -> XrfResult<Vec<u8>> {
  let mut bytes: Vec<u8> = Vec::new();

  for value in [0u32, 0, 3, 0, 0, 3] {
    bytes.extend_from_slice(&value.to_le_bytes());
  }

  chunk(OgfGeometryContainerChunk::CHUNK_ID, &bytes)
}

/// The ids a hierarchy visual links to.
fn new_links(children: &[u32]) -> XrfResult<Vec<u8>> {
  let mut bytes: Vec<u8> = (children.len() as u32).to_le_bytes().to_vec();

  for child in children {
    bytes.extend_from_slice(&child.to_le_bytes());
  }

  chunk(OgfChildrenLinkChunk::CHUNK_ID, &bytes)
}

/// A visuals run of the given visuals, each already framed as its own numbered chunk payload.
fn new_visuals(run: &[Vec<u8>]) -> XrfResult<LevelVisualsChunk> {
  let mut body: Vec<u8> = Vec::new();

  for (index, visual) in run.iter().enumerate() {
    body.extend(chunk(index as u32, visual)?);
  }

  let level: Vec<u8> = chunk(LevelVisualsChunk::CHUNK_ID, &body)?;

  Ok(LevelFile::read_visuals_from_bytes::<XRayByteOrder>(level)?.expect("a run of visuals"))
}

/// One hierarchy linking two drawables, which is the shape every shipped sector has.
fn new_tree() -> XrfResult<LevelVisualsChunk> {
  let mut root: Vec<u8> = new_header(1)?;

  root.extend(new_links(&[1, 2])?);

  let mut first: Vec<u8> = new_header(0)?;

  first.extend(new_container()?);

  let mut second: Vec<u8> = new_header(0)?;

  second.extend(new_container()?);

  new_visuals(&[root, first, second])
}

#[test]
fn test_walks_a_root_to_the_drawables_it_composes() -> XrfResult {
  let composition: LevelSectorComposition = LevelSectorComposition::of(&new_tree()?, 0);

  assert_eq!(composition.drawables, vec![1, 2]);
  assert_eq!(composition.hierarchies, vec![0], "the root draws nothing itself");
  assert_eq!(composition.count_reached(), 3);
  assert!(composition.is_resolved());
  assert!(composition.revisited.is_empty());

  Ok(())
}

#[test]
fn test_a_drawable_root_is_its_own_only_drawable() -> XrfResult {
  let mut only: Vec<u8> = new_header(0)?;

  only.extend(new_container()?);

  let composition: LevelSectorComposition = LevelSectorComposition::of(&new_visuals(&[only])?, 0);

  assert_eq!(composition.drawables, vec![0]);
  assert!(composition.hierarchies.is_empty());

  Ok(())
}

#[test]
fn test_a_link_no_visual_answers_to_is_recorded_rather_than_followed() -> XrfResult {
  let mut root: Vec<u8> = new_header(1)?;

  root.extend(new_links(&[1, 9])?);

  let mut drawable: Vec<u8> = new_header(0)?;

  drawable.extend(new_container()?);

  let composition: LevelSectorComposition = LevelSectorComposition::of(&new_visuals(&[root, drawable])?, 0);

  assert_eq!(composition.drawables, vec![1]);
  assert_eq!(composition.unknown, vec![9]);
  assert!(!composition.is_resolved(), "a level reaching nothing is not resolved");

  Ok(())
}

#[test]
fn test_a_cycle_is_recorded_rather_than_followed_forever() -> XrfResult {
  let mut first: Vec<u8> = new_header(1)?;

  first.extend(new_links(&[1])?);

  let mut second: Vec<u8> = new_header(1)?;

  second.extend(new_links(&[0])?);

  let composition: LevelSectorComposition = LevelSectorComposition::of(&new_visuals(&[first, second])?, 0);

  assert_eq!(composition.hierarchies, vec![0, 1]);
  assert_eq!(composition.revisited, vec![0], "the walk stops rather than looping");
  assert!(composition.drawables.is_empty());

  Ok(())
}

#[test]
fn test_a_root_no_visual_answers_to_reaches_nothing() -> XrfResult {
  let composition: LevelSectorComposition = LevelSectorComposition::of(&new_tree()?, 7);

  assert_eq!(composition.unknown, vec![7]);
  assert!(composition.drawables.is_empty());
  assert_eq!(composition.count_reached(), 1);

  Ok(())
}
