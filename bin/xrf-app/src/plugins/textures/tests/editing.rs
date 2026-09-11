//! Pins what the editor writes: the descriptor form's round trip, where a save lands, and what it refuses.

use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use xrf_db::{ThmBumpMode, ThmFile, ThmFormat, ThmTextureFlag, ThmTextureFlags, ThmTextureType, XRayByteOrder};
use xrf_dds::{DdsEncoding, DdsFile, DdsMipChain, DdsMipFilter, DdsMipmaps, ImageFormat, Quality};
use xrf_job::{JobHandle, JobOutcome};
use xrf_material::fixtures::{FixtureTree, ThmFixture};
use xrf_texture::{GenerateBumpGloss, GenerateBumpOptions, GenerateBumpProcessor, GenerateBumpResult};
use xrf_vfs::{XrayAsset, XrayAssetContainer, XrayLogicalPath};

use crate::core::session::DocumentSessionId;
use crate::plugins::textures::descriptor_form::TextureDescriptorForm;
use crate::plugins::textures::edit_targets::TextureEditTargets;
use crate::plugins::textures::encoding::TextureEncodingFormat;
use crate::plugins::textures::file_stamp::TextureFileStamp;
use crate::plugins::textures::request::{
  TextureDescriptorSave, TextureEncodingSave, TextureSaveTarget, TexturesSaveRequest,
};
use crate::plugins::textures::save::{TextureSaveOutcome, write_save};
use crate::plugins::textures::tests::fixtures::{
  BASE, editing_directory, loose_asset, read_descriptor, source_image, target,
};

/// Move a file's modification time on, so a rewrite a test just made is one the guard can actually see.
///
/// The stamp is size and modification time, and these fixtures rewrite a descriptor with one float changed - same
/// size, and, two statements apart, the same millisecond. What "somebody else rewrote it" means is that the rewrite
/// happened afterwards, so the test says so rather than depending on the clock ticking mid-test.
fn rewritten_later(path: &Path) {
  let file: File = File::options().write(true).open(path).expect("descriptor is writable");
  let modified: SystemTime = file
    .metadata()
    .expect("metadata")
    .modified()
    .expect("modification time");

  file
    .set_modified(modified + Duration::from_secs(1))
    .expect("modification time is writable");
}

#[test]
fn reading_a_descriptor_and_writing_it_back_changes_nothing() {
  // The invariant the whole form rests on. The editor sends fields rather than the file, so applying what it read has
  // to leave every chunk it does not model - the thumbnail, and whatever a later SDK wrote into `extra` - exactly as
  // it was, presence included.
  for fixture in [
    ThmFixture::image(),
    ThmFixture::image().with_bump(ThmBumpMode::Use, "ston\\ston_beton05_bump"),
    ThmFixture::image().with_detail("detail\\detail_grnd_grass", 0.5, &[ThmTextureFlag::BumpDetail]),
    ThmFixture::image().without_bump(),
    ThmFixture::image().without_texture_type(),
    ThmFixture::image().without_texture_param(),
  ] {
    let original: ThmFile = ThmFile::read_from_bytes::<XRayByteOrder>(fixture.to_bytes()).expect("fixture parses");
    let mut applied: ThmFile = original.clone();

    TextureDescriptorForm::read(&original).apply_to(&mut applied);

    assert_eq!(applied, original, "expect a read and write back to be the identity");
    assert_eq!(
      applied.write_to_bytes::<XRayByteOrder>().expect("writes"),
      original.write_to_bytes::<XRayByteOrder>().expect("writes"),
      "expect the same bytes, not merely the same fields"
    );
  }
}

#[test]
fn a_chunk_that_was_absent_is_added_only_when_the_form_asks_for_something() {
  let original: ThmFile =
    ThmFile::read_from_bytes::<XRayByteOrder>(ThmFixture::image().without_bump().to_bytes()).expect("fixture parses");

  assert!(original.bump.is_none(), "expect the fixture to carry no bump chunk");

  // Naming a bump is asking for one, so the chunk appears.
  let mut declared: ThmFile = original.clone();
  let mut form: TextureDescriptorForm = TextureDescriptorForm::read(&original);

  form.bump_name = String::from("ston\\ston_beton05_bump");
  form.bump_mode = u32::from(ThmBumpMode::Use);
  form.apply_to(&mut declared);

  assert_eq!(
    declared.bump.as_ref().map(|bump| bump.name.as_str()),
    Some("ston\\ston_beton05_bump")
  );

  // Changing an unrelated field does not, because a chunk nobody asked for is bytes the file never had.
  let mut untouched: ThmFile = original.clone();
  let mut unrelated: TextureDescriptorForm = TextureDescriptorForm::read(&original);

  unrelated.border_color = 0x00FF_00FF;
  unrelated.apply_to(&mut untouched);

  assert!(untouched.bump.is_none());
}

#[test]
fn the_targets_of_a_texture_name_the_descriptor_beside_it_whether_or_not_one_is_there() {
  let tree: FixtureTree = FixtureTree::new("textures_targets").with_texture(BASE);
  let texture: XrayAsset = loose_asset(tree.root(), "textures\\ston\\ston_beton05.dds");

  let targets: TextureEditTargets = TextureEditTargets::of(&texture, None)
    .expect("targets are readable")
    .expect("a loose texture has targets");

  assert!(targets.texture.path.ends_with("ston_beton05.dds"));
  assert!(targets.descriptor.path.ends_with("ston_beton05.thm"));

  // Which of the two cases this is, is the stamp: the texture is there and the descriptor is not.
  assert!(targets.texture.expected.is_some());
  assert_eq!(
    targets.descriptor.expected, None,
    "expect a texture with no descriptor to be told apart from one whose descriptor a save would replace"
  );
}

#[test]
fn an_archived_texture_has_nothing_to_write() {
  let archived: XrayAsset = XrayAsset::new(
    XrayLogicalPath::new("textures\\ston\\ston_beton05.dds").expect("logical path"),
    XrayAssetContainer::Archive {
      path: PathBuf::from("gamedata.db0"),
    },
  );

  assert_eq!(
    TextureEditTargets::of(&archived, None).expect("targets are readable"),
    None
  );
}

#[test]
fn saving_a_descriptor_writes_it_and_says_where() {
  let root: PathBuf = editing_directory("descriptor");
  let path: PathBuf = root.join("ston_beton05.thm");

  std::fs::write(&path, ThmFixture::image().to_bytes()).expect("descriptor is writable");

  let mut form: TextureDescriptorForm = TextureDescriptorForm::read(&read_descriptor(&path));

  form.texture_type = u32::from(ThmTextureType::Terrain);
  form.detail_name = String::from("detail\\detail_grnd_grass");
  form.detail_scale = 0.25;

  let outcome: TextureSaveOutcome = write_save(
    &JobHandle::inert(),
    &TexturesSaveRequest {
      descriptor: Some(TextureDescriptorSave {
        target: target(&path),
        form,
      }),
      texture: None,
    },
    None,
  )
  .expect("save succeeds");

  assert_eq!(outcome.written, vec![path.display().to_string()]);
  assert_eq!(
    outcome.descriptor_format, None,
    "expect no format sync without a texture"
  );

  let written: ThmFile = read_descriptor(&path);

  assert_eq!(written.texture_type, Some(ThmTextureType::Terrain));
  assert_eq!(written.detail.as_ref().map(|detail| detail.scale), Some(0.25));
}

#[test]
fn saving_a_descriptor_that_is_not_there_yet_creates_one() {
  let root: PathBuf = editing_directory("authored");
  let path: PathBuf = root.join("ston_beton05.thm");

  let outcome: TextureSaveOutcome = write_save(
    &JobHandle::inert(),
    &TexturesSaveRequest {
      descriptor: Some(TextureDescriptorSave {
        target: TextureSaveTarget {
          path: path.display().to_string(),
          expected: None,
        },
        form: TextureDescriptorForm::read(&ThmFile::new_texture()),
      }),
      texture: None,
    },
    None,
  )
  .expect("save succeeds");

  assert_eq!(outcome.written.len(), 1);
  assert_eq!(
    read_descriptor(&path),
    ThmFile::new_texture(),
    "expect an authored descriptor to be the SDK's own defaults"
  );
}

#[test]
fn a_target_that_changed_on_disk_is_refused_rather_than_overwritten() {
  let root: PathBuf = editing_directory("stale");
  let path: PathBuf = root.join("ston_beton05.thm");

  std::fs::write(&path, ThmFixture::image().to_bytes()).expect("descriptor is writable");

  let stamped: TextureSaveTarget = target(&path);
  let before: Vec<u8> = std::fs::read(&path).expect("descriptor is readable");

  // Somebody else rewrote it - an SDK, a converter, another window - after the editor read it.
  std::fs::write(&path, ThmFixture::image().with_virtual_height(0.25).to_bytes()).expect("descriptor is writable");

  rewritten_later(&path);

  let refused = write_save(
    &JobHandle::inert(),
    &TexturesSaveRequest {
      descriptor: Some(TextureDescriptorSave {
        target: stamped,
        form: TextureDescriptorForm::read(&ThmFile::new_texture()),
      }),
      texture: None,
    },
    None,
  );

  assert!(refused.is_err_and(|error| error.contains("changed on disk")));
  assert_ne!(
    std::fs::read(&path).expect("descriptor is readable"),
    before,
    "expect the refusal to leave the other writer's file, not restore ours"
  );
}

#[test]
fn a_file_that_appeared_where_the_editor_saw_none_is_refused_too() {
  let root: PathBuf = editing_directory("appeared");
  let path: PathBuf = root.join("ston_beton05.thm");

  std::fs::write(&path, ThmFixture::image().to_bytes()).expect("descriptor is writable");

  let refused = write_save(
    &JobHandle::inert(),
    &TexturesSaveRequest {
      descriptor: Some(TextureDescriptorSave {
        // The editor read nothing here, so a file now present is a descriptor somebody else authored meanwhile.
        target: TextureSaveTarget {
          path: path.display().to_string(),
          expected: None,
        },
        form: TextureDescriptorForm::read(&ThmFile::new_texture()),
      }),
      texture: None,
    },
    None,
  );

  assert!(refused.is_err_and(|error| error.contains("changed on disk")));
}

#[test]
fn saving_a_texture_writes_its_bytes_and_syncs_the_descriptor_format() {
  let root: PathBuf = editing_directory("texture");
  let texture_path: PathBuf = root.join("ston_beton05.dds");
  let descriptor_path: PathBuf = root.join("ston_beton05.thm");

  std::fs::write(&texture_path, b"placeholder").expect("texture is writable");
  std::fs::write(&descriptor_path, ThmFixture::image().to_bytes()).expect("descriptor is writable");

  let encoded: Vec<u8> = DdsEncoding::new(ImageFormat::BC3RgbaUnorm, Quality::Fast)
    .encode(&DdsMipChain::build(&source_image(16), DdsMipmaps::Disabled).expect("chain"))
    .expect("encode")
    .write_to_bytes()
    .expect("bytes");

  let outcome: TextureSaveOutcome = write_save(
    &JobHandle::inert(),
    &TexturesSaveRequest {
      descriptor: Some(TextureDescriptorSave {
        target: target(&descriptor_path),
        form: TextureDescriptorForm::read(&read_descriptor(&descriptor_path)),
      }),
      texture: Some(TextureEncodingSave {
        session_id: DocumentSessionId::new(),
        target: target(&texture_path),
        format: TextureEncodingFormat::Bc3,
      }),
    },
    Some(encoded),
  )
  .expect("save succeeds");

  // The texture first, then the descriptor: the descriptor is the half that should end up right about the other.
  assert_eq!(
    outcome.written,
    vec![
      texture_path.display().to_string(),
      descriptor_path.display().to_string()
    ]
  );
  assert_eq!(outcome.descriptor_format, Some(u32::from(ThmFormat::Dxt5)));
  assert_eq!(
    read_descriptor(&descriptor_path)
      .texture_param
      .map(|param| param.format),
    Some(ThmFormat::Dxt5)
  );
  assert_eq!(
    DdsFile::read_from_path(&texture_path)
      .expect("written texture parses")
      .metadata()
      .width,
    16
  );
}

#[test]
fn a_bc1_texture_names_the_format_its_own_alpha_flag_says() {
  // `tfDXT1` and `tfADXT1` are one encoder told apart by whether the alpha means anything, so only the descriptor's
  // own flags can say which of them a written BC1 texture is.
  for (has_alpha, expected) in [(false, ThmFormat::Dxt1), (true, ThmFormat::Dxt1Alpha)] {
    let root: PathBuf = editing_directory(if has_alpha { "bc1_alpha" } else { "bc1_opaque" });
    let texture_path: PathBuf = root.join("ston_beton05.dds");
    let descriptor_path: PathBuf = root.join("ston_beton05.thm");

    std::fs::write(&texture_path, b"placeholder").expect("texture is writable");

    let encoded: Vec<u8> = DdsEncoding::new(ImageFormat::BC1RgbaUnorm, Quality::Fast)
      .encode(&DdsMipChain::build(&source_image(16), DdsMipmaps::Disabled).expect("chain"))
      .expect("encode")
      .write_to_bytes()
      .expect("bytes");

    let mut form: TextureDescriptorForm = TextureDescriptorForm::read(&ThmFile::new_texture());

    form.flags = ThmTextureFlags::none().with(ThmTextureFlag::HasAlpha, has_alpha).raw();

    let outcome: TextureSaveOutcome = write_save(
      &JobHandle::inert(),
      &TexturesSaveRequest {
        descriptor: Some(TextureDescriptorSave {
          target: TextureSaveTarget {
            path: descriptor_path.display().to_string(),
            expected: None,
          },
          form,
        }),
        texture: Some(TextureEncodingSave {
          session_id: DocumentSessionId::new(),
          target: target(&texture_path),
          format: TextureEncodingFormat::Bc1,
        }),
      },
      Some(encoded),
    )
    .expect("save succeeds");

    assert_eq!(outcome.descriptor_format, Some(u32::from(expected)));
  }
}

#[test]
fn a_bc7_texture_leaves_the_descriptors_format_alone() {
  // `ETFormat` has no member for BC7, so the descriptor keeps whatever it said and the panel states that an SDK
  // rebuild would return the texture to the named format.
  let root: PathBuf = editing_directory("bc7");
  let texture_path: PathBuf = root.join("ston_beton05.dds");

  std::fs::write(&texture_path, b"placeholder").expect("texture is writable");

  let encoded: Vec<u8> = DdsEncoding::new(ImageFormat::BC7RgbaUnorm, Quality::Fast)
    .encode(&DdsMipChain::build(&source_image(16), DdsMipmaps::Disabled).expect("chain"))
    .expect("encode")
    .write_to_bytes()
    .expect("bytes");

  let outcome: TextureSaveOutcome = write_save(
    &JobHandle::inert(),
    &TexturesSaveRequest {
      descriptor: None,
      texture: Some(TextureEncodingSave {
        session_id: DocumentSessionId::new(),
        target: target(&texture_path),
        format: TextureEncodingFormat::Bc7,
      }),
    },
    Some(encoded),
  )
  .expect("save succeeds");

  assert_eq!(outcome.descriptor_format, None);
}

#[test]
fn saving_a_texture_with_nothing_encoded_is_refused_by_name() {
  let root: PathBuf = editing_directory("unencoded");
  let texture_path: PathBuf = root.join("ston_beton05.dds");

  std::fs::write(&texture_path, b"placeholder").expect("texture is writable");

  let refused = write_save(
    &JobHandle::inert(),
    &TexturesSaveRequest {
      descriptor: None,
      texture: Some(TextureEncodingSave {
        session_id: DocumentSessionId::new(),
        target: target(&texture_path),
        format: TextureEncodingFormat::Bc3,
      }),
    },
    None,
  );

  assert!(refused.is_err_and(|error| error.contains("compare the formats")));
  assert_eq!(
    std::fs::read(&texture_path).expect("texture is readable"),
    b"placeholder",
    "expect a refused save to write nothing at all"
  );
}

#[test]
fn a_stamp_tells_a_missing_file_from_an_unreadable_one() {
  let root: PathBuf = editing_directory("stamp");
  let path: PathBuf = root.join("ston_beton05.thm");

  assert_eq!(TextureFileStamp::read(&path).expect("absent is not an error"), None);

  std::fs::write(&path, b"twelve bytes").expect("file is writable");

  assert_eq!(
    TextureFileStamp::read(&path)
      .expect("present is readable")
      .map(|stamp| stamp.size),
    Some(12)
  );
}

#[test]
fn a_stamp_is_the_size_and_the_modification_time_together() {
  // Both, because a rewrite that happens to land on the same length is exactly the change a size alone would miss.
  let stamp: TextureFileStamp = TextureFileStamp {
    size: 12,
    modified_ms: 1_700_000_000_000,
  };

  assert_ne!(stamp, TextureFileStamp { size: 13, ..stamp });
  assert_ne!(
    stamp,
    TextureFileStamp {
      modified_ms: 1_700_000_000_001,
      ..stamp
    }
  );
}

#[test]
fn a_save_asked_to_stop_before_it_writes_writes_nothing() {
  // The one boundary a save has. Everything before it reads and builds; everything after only publishes, so stopping
  // between two staged writes - a texture whose descriptor still describes the old one - is a state it cannot reach.
  let root: PathBuf = editing_directory("cancelled");
  let texture_path: PathBuf = root.join("ston_beton05.dds");
  let descriptor_path: PathBuf = root.join("ston_beton05.thm");

  std::fs::write(&texture_path, b"placeholder").expect("texture is writable");
  std::fs::write(&descriptor_path, ThmFixture::image().to_bytes()).expect("descriptor is writable");

  let before: Vec<u8> = std::fs::read(&descriptor_path).expect("descriptor is readable");
  let job: JobHandle = JobHandle::inert();

  job.cancel();

  let outcome: TextureSaveOutcome = write_save(
    &job,
    &TexturesSaveRequest {
      descriptor: Some(TextureDescriptorSave {
        target: target(&descriptor_path),
        form: TextureDescriptorForm::read(&ThmFile::new_texture()),
      }),
      texture: Some(TextureEncodingSave {
        session_id: DocumentSessionId::new(),
        target: target(&texture_path),
        format: TextureEncodingFormat::Bc3,
      }),
    },
    Some(vec![0; 128]),
  )
  .expect("a stopped save is a result, not a failure");

  assert_eq!(outcome.outcome, JobOutcome::Cancelled);
  assert!(outcome.written.is_empty());
  assert_eq!(
    outcome.descriptor_format, None,
    "expect no format sync to be claimed for a texture that was not written"
  );
  assert_eq!(std::fs::read(&descriptor_path).expect("descriptor is readable"), before);
  assert_eq!(
    std::fs::read(&texture_path).expect("texture is readable"),
    b"placeholder"
  );
}

#[test]
fn a_save_still_refuses_a_stale_target_before_it_looks_at_cancellation() {
  // Order matters: a stale target is a request the editor got wrong and should hear about, whether or not somebody
  // also pressed stop. Reporting it as a clean cancellation would lose the reason to reload.
  let root: PathBuf = editing_directory("cancelled_stale");
  let path: PathBuf = root.join("ston_beton05.thm");

  std::fs::write(&path, ThmFixture::image().to_bytes()).expect("descriptor is writable");

  let stamped: TextureSaveTarget = target(&path);

  std::fs::write(&path, ThmFixture::image().with_virtual_height(0.25).to_bytes()).expect("descriptor is writable");

  rewritten_later(&path);

  let job: JobHandle = JobHandle::inert();

  job.cancel();

  assert!(
    write_save(
      &job,
      &TexturesSaveRequest {
        descriptor: Some(TextureDescriptorSave {
          target: stamped,
          form: TextureDescriptorForm::read(&ThmFile::new_texture()),
        }),
        texture: None,
      },
      None,
    )
    .is_err_and(|error| error.contains("changed on disk"))
  );
}

#[test]
fn a_bump_generation_asked_to_stop_leaves_neither_half() {
  // Both halves are encoded before either is written, so there is no point at which stopping could leave a `_bump`
  // whose `_bump#` never arrived - which is the pair half of a bumped surface the engine would then substitute for.
  let root: PathBuf = editing_directory("cancelled_bump");
  let job: JobHandle = JobHandle::inert();

  job.cancel();

  let result: GenerateBumpResult = GenerateBumpProcessor::generate(&GenerateBumpOptions {
    job,
    destination: root.join("ston_beton05"),
    height: source_image(16),
    gloss: GenerateBumpGloss::Constant(0.5),
    normal_map: None,
    virtual_height: GenerateBumpOptions::DEFAULT_VIRTUAL_HEIGHT,
    mip_filter: DdsMipFilter::Box,
    quality: Quality::Fast,
  })
  .expect("a stopped generation is a result, not a failure");

  assert_eq!(result.outcome, JobOutcome::Cancelled);
  assert!(!result.bump.exists() && !result.companion.exists());
  assert!(
    result.gloss_power > 0.0,
    "expect the gloss it did measure to be reported, since a dark mask is worth saying even about a stopped run"
  );
}
