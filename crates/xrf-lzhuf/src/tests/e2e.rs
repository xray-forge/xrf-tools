//! End to end: build sample assets, pack them into files, then unpack those files back.
//!
//! Nothing here is committed. The codec takes arbitrary bytes, so its samples carry no format worth
//! preserving on disk and belong in code, where the exact bytes under test are visible and no checkout,
//! editor, or line ending rule can reshape them. Every asset is written under the per-process scratch
//! tree in `target/test-resources/` and read back through the filesystem, which is the leg the in-memory
//! unit tests do not cover.
//!
//! Tests in one binary share that scratch root and run in parallel, so each scopes its files to its own
//! subdirectory.

use std::fs;
use std::path::{Path, PathBuf};

use xrf_test_utils::GeneratedResources;

use crate::{compress, decompress};

const CONFIG_ASSET: &str = "system_fragment.ltx";
const DESCRIPTOR_ASSET: &str = "file_descriptors.bin";

/// A configuration fragment shaped like a packed system config, with the CRLF endings X-Ray configs use.
fn config_asset() -> Vec<u8> {
  [
    "; Fragment shaped like a packed system configuration.",
    "",
    "[wpn_ak74]:identity_immunities",
    "$spawn                 = weapons\\wpn_ak74",
    "class                  = WP_AK74",
    "cform                  = skeleton",
    "visual                 = dynamics\\weapons\\wpn_ak74\\wpn_ak74.ogf",
    "description            = enc_weapons1_wpn-ak74",
    "cost                   = 24000",
    "",
    "[wpn_ak74u]:wpn_ak74",
    "$spawn                 = weapons\\wpn_ak74u",
    "visual                 = dynamics\\weapons\\wpn_ak74u\\wpn_ak74u.ogf",
    "description            = enc_weapons1_wpn-ak74u",
    "cost                   = 18000",
    "",
  ]
  .join("\r\n")
  .into_bytes()
}

/// A file descriptor table: binary fields, backslash paths, and bytes no text encoding would survive.
fn descriptor_asset() -> Vec<u8> {
  const NAMES: [&str; 5] = [
    "gamedata\\configs\\system.ltx",
    "gamedata\\configs\\weapons\\w_ak74.ltx",
    "gamedata\\scripts\\xr_logic.script",
    "gamedata\\textures\\ui\\ui_icon_equipment.dds",
    "gamedata\\meshes\\dynamics\\weapons\\wpn_ak74.ogf",
  ];

  let mut asset: Vec<u8> = Vec::new();

  for (index, name) in NAMES.iter().enumerate() {
    let index: u32 = index as u32;

    asset.extend_from_slice(&((name.len() + 16) as u16).to_le_bytes());
    asset.extend_from_slice(&(4096 * (index + 1)).to_le_bytes());
    asset.extend_from_slice(&(1024 * (index + 1)).to_le_bytes());
    asset.extend_from_slice(&0xDEAD_BEEFu32.to_le_bytes());
    asset.extend_from_slice(name.as_bytes());
    asset.extend_from_slice(&(512 * index).to_le_bytes());
  }

  // "Припять" in windows-1251, then a NUL and a 0xff, none of which are valid UTF-8 together.
  asset.extend_from_slice(&[0xcf, 0xf0, 0xe8, 0xef, 0xff, 0xf2, 0xfc, 0x00, 0xff, 0x00]);

  asset
}

/// Write an asset to the scratch tree, pack it into a second file, and hand back both paths.
///
/// `scope` keeps one test's files clear of every other test's.
fn pack_asset(scope: &str, name: &str, asset: &[u8]) -> (Vec<u8>, PathBuf) {
  let source_path: PathBuf = GeneratedResources::write(&format!("{scope}/{name}"), asset).expect("sample asset writes");
  let source: Vec<u8> = fs::read(&source_path).expect("sample asset reads back");

  assert_eq!(source, asset, "sample asset survives its own trip to disk");

  let packed: Vec<u8> = compress(&source).expect("sample asset packs");
  let packed_path: PathBuf =
    GeneratedResources::write(&format!("{scope}/{name}.lzhuf"), &packed).expect("packed asset writes");

  (source, packed_path)
}

fn unpack_file(path: &Path) -> Vec<u8> {
  let packed: Vec<u8> = fs::read(path).expect("packed file reads back");

  decompress(&packed).expect("packed file unpacks")
}

#[test]
fn packs_and_unpacks_a_configuration_fragment() {
  let scope: &str = "packs_and_unpacks_a_configuration_fragment";
  let (source, packed_path) = pack_asset(scope, CONFIG_ASSET, &config_asset());
  let restored: Vec<u8> = unpack_file(&packed_path);

  assert_eq!(
    restored, source,
    "configuration fragment survives the trip through a file"
  );

  // Line endings are ordinary bytes to this codec, and stay that way across a file.
  let carriage_returns: usize = source.iter().filter(|byte| **byte == b'\r').count();

  assert!(carriage_returns > 0, "the sample exercises CRLF endings");
  assert_eq!(
    restored.windows(2).filter(|pair| pair == b"\r\n").count(),
    carriage_returns,
    "every carriage return still leads a line feed"
  );
}

#[test]
fn packs_and_unpacks_a_binary_descriptor_table() {
  let scope: &str = "packs_and_unpacks_a_binary_descriptor_table";
  let (source, packed_path) = pack_asset(scope, DESCRIPTOR_ASSET, &descriptor_asset());
  let restored: Vec<u8> = unpack_file(&packed_path);

  assert_eq!(restored, source, "descriptor table survives the trip through a file");

  assert!(
    source.contains(&0x00) && source.contains(&0xff),
    "the sample is not text"
  );
  assert!(
    String::from_utf8(restored).is_err(),
    "the sample stays invalid UTF-8, so nothing re-encoded it"
  );
}

#[test]
fn packs_each_sample_asset_into_its_own_file() {
  let scope: &str = "packs_each_sample_asset_into_its_own_file";
  let (config_source, config_path) = pack_asset(scope, CONFIG_ASSET, &config_asset());
  let (descriptor_source, descriptor_path) = pack_asset(scope, DESCRIPTOR_ASSET, &descriptor_asset());

  assert_ne!(config_path, descriptor_path, "assets pack to separate files");

  // Read both back after both were written, so one packing run cannot disturb the other's output.
  assert_eq!(unpack_file(&config_path), config_source);
  assert_eq!(unpack_file(&descriptor_path), descriptor_source);
}

#[test]
fn packed_configuration_is_smaller_than_its_source() {
  let scope: &str = "packed_configuration_is_smaller_than_its_source";
  let (source, packed_path) = pack_asset(scope, CONFIG_ASSET, &config_asset());
  let packed_size: u64 = fs::metadata(&packed_path).expect("packed file is on disk").len();

  assert!(
    packed_size < source.len() as u64,
    "packing a repetitive config should shrink it, got {packed_size} from {}",
    source.len()
  );
}

#[test]
fn unpacking_a_truncated_packed_file_fails() {
  let scope: &str = "unpacking_a_truncated_packed_file_fails";
  let (_, packed_path) = pack_asset(scope, CONFIG_ASSET, &config_asset());
  let packed: Vec<u8> = fs::read(&packed_path).expect("packed file reads back");
  let truncated_path: PathBuf = GeneratedResources::write(
    &format!("{scope}/{CONFIG_ASSET}.truncated.lzhuf"),
    &packed[..packed.len() / 2],
  )
  .expect("truncated file writes");

  // The header still declares the full size, so the stream runs out before it is produced.
  assert!(
    decompress(&fs::read(&truncated_path).expect("truncated file reads back")).is_err(),
    "a half written packed file must not unpack into plausible bytes"
  );
}
