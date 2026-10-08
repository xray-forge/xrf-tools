use crate::level::reduced_textures::ReducedTextures;

#[test]
fn reduces_a_reference_holding_a_listed_part_whatever_its_case() {
  let reduced: ReducedTextures = ReducedTextures::new(["Ston_", "wood\\fence"]);

  assert!(reduced.is_reduced("ston\\ston_kirpich_01"));
  assert!(reduced.is_reduced("WOOD\\Fence_old"));
  assert!(!reduced.is_reduced("terrain\\terrain_zaton"));
}

#[test]
fn reduces_nothing_without_a_list() {
  assert!(!ReducedTextures::default().is_reduced("ston\\ston_kirpich_01"));
}
