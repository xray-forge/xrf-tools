//! Equipment extraction writes one icon per section name, so the name must not be able to leave the destination.

use std::fs;
use std::path::{Path, PathBuf};

use image::{Rgba, RgbaImage};
use xrf_dds::ImageFormat;
use xrf_error::XrfError;
use xrf_ltx::Ltx;
use xrf_test_utils::GeneratedResources;

use crate::equipment::{EquipmentSpriteUnpackOptions, EquipmentSpriteUnpacker};

const SENTINEL: &[u8] = b"sentinel";

fn create_root(name: &str) -> PathBuf {
  let root: PathBuf = GeneratedResources::create_dir(&format!("equipment-unpack/{name}")).expect("test root");

  fs::write(root.join("victim.dds"), SENTINEL).expect("sentinel beside the destination");

  root
}

/// One icon per section, laid out left to right in grid order.
fn options_for(root: &Path, sections: &[&str]) -> EquipmentSpriteUnpackOptions {
  let mut ltx: Ltx = Ltx::new();

  for (index, section) in sections.iter().enumerate() {
    ltx
      .with_section(*section)
      .set("$inventory_icon", "true")
      .set("inv_grid_x", index.to_string())
      .set("inv_grid_y", "0")
      .set("inv_grid_width", "1")
      .set("inv_grid_height", "1");
  }

  EquipmentSpriteUnpackOptions {
    job: Default::default(),
    ltx,
    source: RgbaImage::from_pixel(50 * sections.len() as u32, 50, Rgba([255, 255, 255, 255])),
    output: Default::default(),
    output_path: root.join("out"),
    dds_compression_format: ImageFormat::BC3RgbaUnorm,
    link_policy: Default::default(),
  }
}

fn list_output(root: &Path) -> Vec<PathBuf> {
  fs::read_dir(root.join("out"))
    .map(|entries| entries.flatten().map(|entry| entry.path()).collect())
    .unwrap_or_default()
}

#[test]
fn refuses_a_section_name_that_leaves_the_destination() {
  let absolute: String = GeneratedResources::path("equipment-unpack/absolute/victim")
    .display()
    .to_string();

  for (index, section) in [
    "../victim",
    "..\\victim",
    "nested/../../victim",
    "c:victim",
    absolute.as_str(),
  ]
  .into_iter()
  .enumerate()
  {
    let root: PathBuf = create_root(&format!("escape-{index}"));

    // A valid icon is listed first, so a run that wrote before it checked would leave it behind.
    let result = EquipmentSpriteUnpacker::unpack_sprites(options_for(&root, &["wpn_valid", section]));

    assert!(
      matches!(result, Err(XrfError::Invalid { .. })),
      "section '{section}' must be refused by name"
    );
    assert_eq!(
      fs::read(root.join("victim.dds")).expect("sentinel"),
      SENTINEL,
      "'{section}' overwrote the sentinel"
    );
    assert!(list_output(&root).is_empty(), "'{section}' left earlier icons behind");
  }
}

#[test]
fn writes_each_icon_under_its_section_name_as_spelled() {
  let root: PathBuf = create_root("valid");

  EquipmentSpriteUnpacker::unpack_sprites(options_for(&root, &["wpn_AK74", "wpn ak74 (old)"])).expect("unpack");

  assert!(root.join("out").join("wpn_AK74.dds").is_file());
  assert!(root.join("out").join("wpn ak74 (old).dds").is_file());
  assert_eq!(fs::read(root.join("victim.dds")).expect("sentinel"), SENTINEL);
}
