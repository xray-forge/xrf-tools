use std::fmt::Display;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use image::{Rgba, RgbaImage};
use xrf_dds::{DdsFile, ImageFormat};
use xrf_job::{JobHandle, JobOutcome};
use xrf_ltx::Ltx;
use xrf_output::{Output, OutputChannel, OutputOptions, OutputVerbosity};
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

use super::{PackEquipmentOptions, PackEquipmentProcessor};
use crate::image_file::{UI_MIPMAPS, save_image_as_ui_dds};

struct CancelOnOutput {
  job: JobHandle,
  channel: OutputChannel,
}

impl Output for CancelOnOutput {
  fn write(&self, channel: OutputChannel, _: &dyn Display) {
    if channel == self.channel {
      self.job.cancel();
    }
  }
}

fn options(name: &str, icons: u32, existing: bool) -> PackEquipmentOptions {
  let root: PathBuf = build_absolute_generated_test_resource_path(&format!("equipment-pack/{name}/{existing}"));
  if root.exists() {
    fs::remove_dir_all(&root).unwrap();
  }
  fs::create_dir_all(&root).unwrap();

  let mut ltx: Ltx = Ltx::new();
  let icon: RgbaImage = RgbaImage::from_pixel(50, 50, Rgba([255, 255, 255, 255]));
  for index in 0..icons {
    ltx
      .with_section(format!("icon_{index}"))
      .set("$inventory_icon", "true")
      .set("inv_grid_x", index.to_string())
      .set("inv_grid_y", "0")
      .set("inv_grid_width", "1")
      .set("inv_grid_height", "1");
    icon.save(root.join(format!("icon_{index}.png"))).unwrap();
  }

  let output_path: PathBuf = root.join("atlas.dds");
  if existing {
    let sentinel: RgbaImage = RgbaImage::from_pixel(52, 52, Rgba([255, 255, 255, 255]));
    save_image_as_ui_dds(&output_path, &sentinel, ImageFormat::BC3RgbaUnorm, UI_MIPMAPS).unwrap();
  }

  PackEquipmentOptions {
    job: JobHandle::default(),
    ltx,
    source: root,
    output: OutputOptions::default(),
    output_path,
    gamedata: None,
    dds_compression_format: ImageFormat::BC3RgbaUnorm,
    is_strict: true,
  }
}

fn cancel_on(options: &mut PackEquipmentOptions, channel: OutputChannel) {
  options.output = OutputOptions::new(
    Arc::new(CancelOnOutput {
      job: options.job.clone(),
      channel,
    }),
    OutputVerbosity::Verbose,
  );
}

fn assert_cancelled_without_publication(options: PackEquipmentOptions, packed: u32, skipped: u32) {
  let path: PathBuf = options.output_path.clone();
  let before: Option<Vec<u8>> = path.exists().then(|| fs::read(&path).unwrap());
  let result = PackEquipmentProcessor::pack_sprites(options).unwrap();

  assert_eq!(result.outcome, JobOutcome::Cancelled);
  assert_eq!(result.packed_count, packed);
  assert_eq!(result.skipped_count, skipped);
  match before {
    Some(bytes) => assert_eq!(fs::read(path).unwrap(), bytes),
    None => assert!(!path.exists()),
  }
}

#[test]
fn pre_cancelled_pack_preserves_destination() {
  for existing in [false, true] {
    let options = options("pre-cancelled", 1, existing);
    options.job.cancel();
    assert_cancelled_without_publication(options, 0, 0);
  }
}

#[test]
fn mid_pack_cancellation_preserves_destination() {
  for existing in [false, true] {
    let mut options = options("mid-pack", 2, existing);
    // The icon message is emitted after decoding and before drawing the first icon.
    cancel_on(&mut options, OutputChannel::Verbose);
    assert_cancelled_without_publication(options, 1, 0);
  }
}

#[test]
fn cancellation_during_final_icon_preserves_destination() {
  for existing in [false, true] {
    let mut options = options("final-icon", 1, existing);
    cancel_on(&mut options, OutputChannel::Verbose);
    assert_cancelled_without_publication(options, 1, 0);
  }
}

#[test]
fn cancellation_precedes_strict_missing_icon_validation() {
  for existing in [false, true] {
    let mut options = options("cancelled-missing-icon", 1, existing);
    fs::remove_file(options.source.join("icon_0.png")).unwrap();
    cancel_on(&mut options, OutputChannel::Warning);
    assert_cancelled_without_publication(options, 0, 1);
  }
}

#[test]
fn cancellation_during_shape_warning_preserves_destination() {
  let mut options = options("shape-warning", 2, true);
  cancel_on(&mut options, OutputChannel::Warning);
  assert_cancelled_without_publication(options, 2, 0);
}

#[test]
fn completed_pack_publishes_every_icon() {
  for existing in [false, true] {
    let options = options("completed", 2, existing);
    let path: PathBuf = options.output_path.clone();
    let result = PackEquipmentProcessor::pack_sprites(options).unwrap();
    assert_eq!(result.outcome, JobOutcome::Completed);
    assert_eq!(result.packed_count, 2);
    assert_eq!(result.skipped_count, 0);

    let image = DdsFile::read_from_path(&path).unwrap().decode_rgba(0).unwrap();
    assert_eq!(image.dimensions(), (100, 52));
    for (_, y, pixel) in image.enumerate_pixels() {
      if y < 50 {
        assert_eq!(pixel.0, [255, 255, 255, 255]);
      } else {
        assert_eq!(pixel.0[3], 0);
      }
    }
  }
}

#[test]
fn strict_missing_icon_error_preserves_destination() {
  for existing in [false, true] {
    let options = options("missing-icon", 1, existing);
    fs::remove_file(options.source.join("icon_0.png")).unwrap();
    let path: PathBuf = options.output_path.clone();
    let before: Option<Vec<u8>> = path.exists().then(|| fs::read(&path).unwrap());

    assert!(PackEquipmentProcessor::pack_sprites(options).is_err());
    match before {
      Some(bytes) => assert_eq!(fs::read(path).unwrap(), bytes),
      None => assert!(!path.exists()),
    }
  }
}
