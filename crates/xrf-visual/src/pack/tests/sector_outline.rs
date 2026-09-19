//! What a sector is and where, taken from what its visuals declare rather than from any geometry.

use xrf_level::LevelVisualsChunk;

use crate::data::sector_outline::SectorOutline;
use crate::data::visual_bounds::VisualBounds;
use crate::pack::tests::level_fixtures::{new_drawable_at, new_hierarchy, new_visuals};

#[test]
fn test_reaches_the_drawables_its_root_composes() {
  let run: LevelVisualsChunk = new_visuals(&[
    new_hierarchy(&[1, 2]),
    new_drawable_at(1, [0.0, 0.0, 0.0], [10.0, 4.0, 10.0]),
    new_drawable_at(1, [20.0, 0.0, 0.0], [30.0, 4.0, 10.0]),
  ]);

  let outline: SectorOutline = SectorOutline::of(&run, 3, 0);

  assert_eq!(outline.sector, 3);
  assert_eq!(outline.root, 0);
  assert_eq!(outline.drawables, 2);
}

#[test]
fn test_spans_every_drawable_it_reaches() {
  let run: LevelVisualsChunk = new_visuals(&[
    new_hierarchy(&[1, 2]),
    new_drawable_at(1, [0.0, 0.0, 0.0], [10.0, 4.0, 10.0]),
    new_drawable_at(1, [20.0, -2.0, 0.0], [30.0, 4.0, 10.0]),
  ]);

  let bounds: VisualBounds = SectorOutline::of(&run, 0, 0).bounds.expect("a sector of two drawables");

  assert_eq!(bounds.bounding_box.min.x, 0.0);
  assert_eq!(bounds.bounding_box.max.x, 30.0);
  assert_eq!(bounds.bounding_box.min.y, -2.0);
}

// The camera a viewer streams against is in renderer space, so an outline in the engine's would mirror what loads
// about the level's own z axis - which looks entirely plausible until it is flown.
#[test]
fn test_declares_its_extent_in_renderer_space() {
  let run: LevelVisualsChunk = new_visuals(&[
    new_hierarchy(&[1]),
    new_drawable_at(1, [0.0, 0.0, 10.0], [1.0, 1.0, 20.0]),
  ]);

  let bounds: VisualBounds = SectorOutline::of(&run, 0, 0).bounds.expect("a sector of one drawable");

  assert_eq!(bounds.bounding_box.min.z, -20.0, "z is mirrored on the way in");
  assert_eq!(bounds.bounding_box.max.z, -10.0);
}

// Residency never reads a sector with no extent, so a sector that reaches nothing has to say so rather than claim
// the origin - which would pull every camera near the middle of the level towards it.
#[test]
fn test_a_sector_reaching_nothing_declares_no_extent() {
  let run: LevelVisualsChunk = new_visuals(&[new_hierarchy(&[])]);
  let outline: SectorOutline = SectorOutline::of(&run, 0, 0);

  assert_eq!(outline.drawables, 0);
  assert!(outline.bounds.is_none());
}

#[test]
fn test_a_root_no_visual_answers_to_declares_no_extent() {
  let run: LevelVisualsChunk = new_visuals(&[new_hierarchy(&[])]);

  assert!(SectorOutline::of(&run, 0, 9).bounds.is_none());
}

#[test]
fn test_the_level_spans_every_sector_that_declares_anything() {
  let run: LevelVisualsChunk = new_visuals(&[
    new_hierarchy(&[1]),
    new_drawable_at(1, [0.0, 0.0, 0.0], [10.0, 4.0, 10.0]),
    new_hierarchy(&[3]),
    new_drawable_at(1, [100.0, 0.0, 0.0], [110.0, 4.0, 10.0]),
    new_hierarchy(&[]),
  ]);

  let outlines: Vec<SectorOutline> = vec![
    SectorOutline::of(&run, 0, 0),
    SectorOutline::of(&run, 1, 2),
    SectorOutline::of(&run, 2, 4),
  ];

  let bounds: VisualBounds = SectorOutline::merge_bounds(&outlines).expect("a level of two drawn sectors");

  assert_eq!(bounds.bounding_box.min.x, 0.0);
  assert_eq!(bounds.bounding_box.max.x, 110.0);
}

#[test]
fn test_a_level_that_declares_nothing_spans_nothing() {
  assert!(SectorOutline::merge_bounds(&[]).is_none());
}
