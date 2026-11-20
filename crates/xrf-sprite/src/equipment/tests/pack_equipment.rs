use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use image::{Rgba, RgbaImage};
use xrf_dds::{DdsFile, ImageFormat};
use xrf_job::{JobHandle, JobOutcome, JobProgress, ProgressSink};
use xrf_ltx::{Ltx, LtxSectionWrite};
use xrf_test_utils::GeneratedResources;

use crate::equipment::{EquipmentSpritePackOptions, EquipmentSpritePacker, InventorySpriteDescriptor};
use crate::job_phases::SPRITE_PHASE_PACK_EQUIPMENT;
use crate::report::{SpriteNotice, SpriteNoticeRecorder};
use xrf_texture::{ImageFileReshape, ImageFileWriter};

/// Cancels its job as soon as the pack has started on its first icon, while that icon is still being read and drawn.
#[derive(Default)]
struct CancelOnFirstIcon {
  job: OnceLock<JobHandle>,
}

impl ProgressSink for CancelOnFirstIcon {
  fn report(&self, progress: &JobProgress) {
    let is_on_an_icon: bool = progress
      .levels
      .iter()
      .any(|level| level.id == SPRITE_PHASE_PACK_EQUIPMENT && level.completed > 0);

    if is_on_an_icon {
      self.job.get().expect("the sink is given its job").cancel();
    }
  }
}

fn options(name: &str, icons: u32, existing: bool) -> EquipmentSpritePackOptions {
  let root: PathBuf = GeneratedResources::create_dir(&format!("equipment-pack/{name}/{existing}")).unwrap();

  let mut ltx: Ltx = Ltx::new();
  let icon: RgbaImage = RgbaImage::from_pixel(50, 50, Rgba([255, 255, 255, 255]));
  for index in 0..icons {
    ltx
      .with_section(format!("icon_{index}"))
      .set("$inventory_icon", "true")
      .set("inv_grid_x", index)
      .set("inv_grid_y", "0")
      .set("inv_grid_width", "1")
      .set("inv_grid_height", "1");
    icon.save(root.join(format!("icon_{index}.png"))).unwrap();
  }

  let output_path: PathBuf = root.join("atlas.dds");
  if existing {
    let sentinel: RgbaImage = RgbaImage::from_pixel(52, 52, Rgba([255, 255, 255, 255]));
    ImageFileWriter::write_dds(
      &output_path,
      &sentinel,
      ImageFormat::BC3RgbaUnorm,
      ImageFileWriter::UI_MIPMAPS,
    )
    .unwrap();
  }

  EquipmentSpritePackOptions {
    listener: None,
    job: JobHandle::default(),
    ltx,
    source: root,
    output_path,
    gamedata: None,
    dds_compression_format: ImageFormat::BC3RgbaUnorm,
    is_strict: true,
  }
}

fn cancel_on_first_icon(options: &mut EquipmentSpritePackOptions) {
  let sink: Arc<CancelOnFirstIcon> = Arc::new(CancelOnFirstIcon::default());

  options.job = JobHandle::with_interval(Arc::<CancelOnFirstIcon>::clone(&sink), Duration::ZERO);

  assert!(
    sink.job.set(options.job.clone()).is_ok(),
    "the sink is given its job once"
  );
}

/// Packs, cancelling the job when `is_cancelling` picks a notice, and checks nothing was published.
fn assert_cancelled_without_publication(
  mut options: EquipmentSpritePackOptions,
  packed: u32,
  skipped: u32,
  is_cancelling: impl Fn(&SpriteNotice) -> bool + Send + Sync + 'static,
) {
  let path: PathBuf = options.output_path.clone();
  let before: Option<Vec<u8>> = path.exists().then(|| fs::read(&path).unwrap());
  let job: JobHandle = options.job.clone();

  options.listener = Some(Arc::new(move |notice: &SpriteNotice| {
    if is_cancelling(notice) {
      job.cancel();
    }
  }));

  let result = EquipmentSpritePacker::pack_sprites(options).unwrap();

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
    assert_cancelled_without_publication(options, 0, 0, |_| false);
  }
}

#[test]
fn mid_pack_cancellation_preserves_destination() {
  for existing in [false, true] {
    let mut options = options("mid-pack", 2, existing);
    cancel_on_first_icon(&mut options);
    assert_cancelled_without_publication(options, 1, 0, |_| false);
  }
}

#[test]
fn cancellation_during_final_icon_preserves_destination() {
  for existing in [false, true] {
    let mut options = options("final-icon", 1, existing);
    cancel_on_first_icon(&mut options);
    assert_cancelled_without_publication(options, 1, 0, |_| false);
  }
}

#[test]
fn cancellation_precedes_strict_missing_icon_validation() {
  for existing in [false, true] {
    let options = options("cancelled-missing-icon", 1, existing);
    fs::remove_file(options.source.join("icon_0.png")).unwrap();
    assert_cancelled_without_publication(options, 0, 1, |notice| {
      matches!(notice, SpriteNotice::IconUnreadable { .. })
    });
  }
}

#[test]
fn cancellation_during_shape_notice_preserves_destination() {
  let options = options("shape-notice", 2, true);
  assert_cancelled_without_publication(options, 2, 0, |notice| {
    matches!(notice, SpriteNotice::SheetReshaped { .. })
  });
}

#[test]
fn notices_a_sheet_written_over_one_of_another_shape() {
  let mut options = options("reshaped", 2, true);
  let path: PathBuf = options.output_path.clone();
  let recorder: SpriteNoticeRecorder = SpriteNoticeRecorder::default();

  options.listener = recorder.new_listener();

  EquipmentSpritePacker::pack_sprites(options).unwrap();

  assert_eq!(
    recorder.list_notices(),
    [SpriteNotice::SheetReshaped {
      path,
      width: 100,
      height: 52,
      mipmap_levels: ImageFileWriter::UI_MIPMAP_LEVELS,
      replaced: ImageFileReshape::Reshaped {
        width: 52,
        height: 52,
        mipmap_levels: ImageFileWriter::UI_MIPMAP_LEVELS,
      },
    }]
  );
}

#[test]
fn completed_pack_publishes_every_icon() {
  for existing in [false, true] {
    let options = options("completed", 2, existing);
    let path: PathBuf = options.output_path.clone();
    let result = EquipmentSpritePacker::pack_sprites(options).unwrap();
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
fn refuses_a_custom_icon_path_it_cannot_resolve() {
  // A marker without its separator, or a relative path in a config read from no directory: an error, not a panic.
  for (custom, is_from_directory) in [("~ui\\icon", true), ("#ui\\icon", true), ("ui\\icon", false)] {
    let mut options = options("unresolvable-custom-icon", 1, false);
    options.ltx.with_section("icon_0").set("$inventory_icon_path", custom);
    options.ltx.directory = is_from_directory.then(|| options.source.clone());

    let descriptors: Vec<InventorySpriteDescriptor> = InventorySpriteDescriptor::new_list_from_ltx(&options.ltx);

    assert!(
      EquipmentSpritePacker::read_sprite_path(&options, &descriptors[0]).is_err(),
      "{custom}"
    );
  }
}

#[test]
fn strict_missing_icon_error_preserves_destination() {
  for existing in [false, true] {
    let mut options = options("missing-icon", 1, existing);
    fs::remove_file(options.source.join("icon_0.png")).unwrap();
    let path: PathBuf = options.output_path.clone();
    let before: Option<Vec<u8>> = path.exists().then(|| fs::read(&path).unwrap());
    let recorder: SpriteNoticeRecorder = SpriteNoticeRecorder::default();

    options.listener = recorder.new_listener();

    assert!(EquipmentSpritePacker::pack_sprites(options).is_err());
    // The refusal names the section; the notice handed out before it says why the icon could not be read.
    assert!(matches!(
      recorder.list_notices().as_slice(),
      [SpriteNotice::IconUnreadable { section, .. }] if section == "icon_0"
    ));
    match before {
      Some(bytes) => assert_eq!(fs::read(path).unwrap(), bytes),
      None => assert!(!path.exists()),
    }
  }
}
