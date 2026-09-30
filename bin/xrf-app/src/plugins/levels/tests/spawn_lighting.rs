use std::cell::Cell;
use std::sync::Arc;

use xrf_level::{LevelCformGeometry, LevelCformTracer};
use xrf_visual::HemiEstimator;

use crate::plugins::levels::state::LevelSpawnLighting;

fn estimator(built: &Cell<u32>) -> Result<Arc<HemiEstimator>, String> {
  built.set(built.get() + 1);

  let geometry: LevelCformGeometry = LevelCformGeometry::new(Vec::new(), Vec::new()).map_err(|it| it.to_string())?;

  Ok(Arc::new(HemiEstimator::new(LevelCformTracer::new(&geometry), &[])))
}

fn names(names: &[&str]) -> Vec<String> {
  names.iter().map(|it| String::from(*it)).collect()
}

#[test]
fn holds_the_estimator_until_every_expected_visual_is_described() -> Result<(), String> {
  let lighting: LevelSpawnLighting = LevelSpawnLighting::new();
  let built: Cell<u32> = Cell::new(0);

  lighting.expect(&names(&["crate", "lamp"]))?;
  lighting.get_or_build(|| estimator(&built))?;
  lighting.get_or_build(|| estimator(&built))?;
  lighting.note_described(&names(&["crate"]))?;

  assert_eq!(built.get(), 1);
  assert!(lighting.is_held());

  lighting.note_described(&names(&["lamp"]))?;

  assert!(!lighting.is_held());

  // Asked again after it went, it is built again rather than refused.
  lighting.get_or_build(|| estimator(&built))?;

  assert_eq!(built.get(), 2);

  Ok(())
}

#[test]
fn keeps_a_failure_rather_than_trying_again_while_visuals_are_pending() -> Result<(), String> {
  let lighting: LevelSpawnLighting = LevelSpawnLighting::new();
  let tried: Cell<u32> = Cell::new(0);
  let fail = || {
    tried.set(tried.get() + 1);

    Err(String::from("no collision form"))
  };

  lighting.expect(&names(&["crate"]))?;

  assert_eq!(
    lighting.get_or_build(fail).err(),
    Some(String::from("no collision form"))
  );
  assert!(lighting.get_or_build(fail).is_err());
  assert_eq!(tried.get(), 1);

  Ok(())
}
