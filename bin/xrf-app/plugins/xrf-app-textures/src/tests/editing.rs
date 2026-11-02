//! Pins what the editor writes: the descriptor form's round trip, where a save lands, and what it refuses.

use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, SystemTime};

use xrf_app_core::assets::AssetMountState;
use xrf_app_core::session::{SessionDerived, SessionId, SessionScopes, Sessions};
use xrf_chunk::XRayByteOrder;
use xrf_dds::{DdsEncoding, DdsFile, DdsMipChain, DdsMipmaps, ImageFormat, Quality};
use xrf_job::{JobHandle, JobOutcome};
use xrf_material::fixtures::{FixtureTree, ThmFixture};
use xrf_texture::TextureEncodingFormat;
use xrf_thm::{ThmBumpMode, ThmFile, ThmFormat, ThmTextureFlag, ThmTextureFlags, ThmTextureType};
use xrf_vfs::{XrayAsset, XrayAssetContainer, XrayInputRecord, XrayLogicalPath, XrayRevision, XraySearch};

use crate::edit::{
  TextureDescriptorForm, TextureDescriptorSave, TextureEditTargets, TextureSaveOutcome, TextureSaveTarget,
  TextureSaver, TextureWrite,
};
use crate::tests::fixtures::{BASE, editing_directory, loose_asset, read_descriptor, roots_of, source_image, target};

/// Move a file's modification time on, so a rewrite a test just made is one the guard can actually see.
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

/// A candidate's bytes, encoded as `format` names.
fn encoded(format: ImageFormat) -> Vec<u8> {
  DdsEncoding::new(format, Quality::Fast)
    .encode(&DdsMipChain::build(&source_image(16), DdsMipmaps::Disabled).expect("chain"))
    .expect("encode")
    .write_to_bytes()
    .expect("bytes")
}

/// The descriptor half of a save, as the editor read it at `path`.
fn descriptor_save(path: &Path, form: TextureDescriptorForm) -> TextureDescriptorSave {
  TextureDescriptorSave {
    target: target(path),
    form,
  }
}

/// The texture half of a save, a held `format` candidate written over the texture at `path`.
fn texture_write(path: &Path, format: TextureEncodingFormat, bytes: Vec<u8>) -> TextureWrite {
  TextureWrite {
    path: path.to_path_buf(),
    expected: target(path).expected,
    format,
    bytes,
  }
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

  let outcome: TextureSaveOutcome = TextureSaver::save(
    &AssetMountState::new(),
    &JobHandle::inert(),
    &descriptor_save(&path, form),
    None,
  )
  .expect("save succeeds");

  assert_eq!(outcome.written, vec![path.display().to_string()]);

  let written: ThmFile = read_descriptor(&path);

  assert_eq!(written.texture_type, Some(ThmTextureType::Terrain));
  assert_eq!(written.detail.as_ref().map(|detail| detail.scale), Some(0.25));
}

#[test]
fn saving_a_descriptor_that_is_not_there_yet_creates_one() {
  let root: PathBuf = editing_directory("authored");
  let path: PathBuf = root.join("ston_beton05.thm");

  let outcome: TextureSaveOutcome = TextureSaver::save(
    &AssetMountState::new(),
    &JobHandle::inert(),
    &TextureDescriptorSave {
      target: TextureSaveTarget {
        path: path.display().to_string(),
        expected: None,
      },
      form: TextureDescriptorForm::read(&ThmFile::new_texture()),
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
fn a_save_into_a_mounted_tree_commits_so_its_retained_descriptor_and_sessions_follow() {
  let tree: FixtureTree = FixtureTree::new("textures_committed")
    .with_texture(BASE)
    .with_descriptor(BASE, &ThmFixture::image());
  let assets: AssetMountState = AssetMountState::new();
  let search: XraySearch = assets.search(&roots_of(&tree)).expect("the tree mounts");
  let descriptor: XrayAsset = search
    .find("textures\\ston\\ston_beton05.thm")
    .expect("the lookup succeeds")
    .expect("the descriptor is mounted");
  let path: PathBuf = descriptor.to_physical_path().expect("the descriptor is loose");
  let parses: AtomicU32 = AtomicU32::new(0);
  let read = |search: &XraySearch| -> Arc<Vec<u8>> {
    search
      .read_parsed(&descriptor, |bytes| {
        parses.fetch_add(1, Ordering::Relaxed);

        Ok(bytes)
      })
      .expect("the descriptor reads")
  };
  let sessions: Sessions<&'static str> = Sessions::new("test", &SessionScopes::default())
    .with_derivation(&assets, |_, _| {
      Ok(SessionDerived::new("derived", XrayInputRecord::new()))
    });
  let inputs: XrayInputRecord = XrayInputRecord::new();
  let id: SessionId = SessionId::generate();

  read(&search.clone().with_recording(&inputs));
  sessions
    .begin_fixture_open(id)
    .expect("the session opens")
    .commit_fixture(SessionDerived::new("opened", inputs), XrayRevision::MOUNTED)
    .expect("the session commits");

  let mut form: TextureDescriptorForm = TextureDescriptorForm::read(&read_descriptor(&path));

  form.texture_type = u32::from(ThmTextureType::Terrain);

  let outcome: TextureSaveOutcome =
    TextureSaver::save(&assets, &JobHandle::inert(), &descriptor_save(&path, form), None).expect("save succeeds");
  let after: XraySearch = assets.search(&roots_of(&tree)).expect("the tree is found");
  let followed = sessions.require(id).expect("the session is held");

  assert_eq!(outcome.written, vec![path.display().to_string()]);
  assert_ne!(
    assets.get_vfs().get_revision(),
    XrayRevision::MOUNTED,
    "the save moved the revision"
  );
  assert_eq!(
    *read(&after),
    std::fs::read(&path).expect("descriptor is readable"),
    "the next read parses what was written"
  );
  assert_eq!(parses.load(Ordering::Relaxed), 2);
  assert_eq!(read_descriptor(&path).texture_type, Some(ThmTextureType::Terrain));
  assert_eq!(**followed, "derived");
  assert_eq!(followed.revision, assets.get_vfs().get_revision());
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

  let refused = TextureSaver::save(
    &AssetMountState::new(),
    &JobHandle::inert(),
    &TextureDescriptorSave {
      target: stamped,
      form: TextureDescriptorForm::read(&ThmFile::new_texture()),
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

  let refused = TextureSaver::save(
    &AssetMountState::new(),
    &JobHandle::inert(),
    &TextureDescriptorSave {
      // The editor read nothing here, so a file now present is a descriptor somebody else authored meanwhile.
      target: TextureSaveTarget {
        path: path.display().to_string(),
        expected: None,
      },
      form: TextureDescriptorForm::read(&ThmFile::new_texture()),
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

  let outcome: TextureSaveOutcome = TextureSaver::save(
    &AssetMountState::new(),
    &JobHandle::inert(),
    &descriptor_save(
      &descriptor_path,
      TextureDescriptorForm::read(&read_descriptor(&descriptor_path)),
    ),
    Some(texture_write(
      &texture_path,
      TextureEncodingFormat::Bc3,
      encoded(ImageFormat::BC3RgbaUnorm),
    )),
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

    let mut form: TextureDescriptorForm = TextureDescriptorForm::read(&ThmFile::new_texture());

    form.flags = ThmTextureFlags::none().with(ThmTextureFlag::HasAlpha, has_alpha).raw();

    TextureSaver::save(
      &AssetMountState::new(),
      &JobHandle::inert(),
      &TextureDescriptorSave {
        target: TextureSaveTarget {
          path: descriptor_path.display().to_string(),
          expected: None,
        },
        form,
      },
      Some(texture_write(
        &texture_path,
        TextureEncodingFormat::Bc1,
        encoded(ImageFormat::BC1RgbaUnorm),
      )),
    )
    .expect("save succeeds");

    assert_eq!(
      read_descriptor(&descriptor_path)
        .texture_param
        .map(|param| param.format),
      Some(expected)
    );
  }
}

#[test]
fn a_bc7_texture_leaves_the_descriptors_format_alone() {
  // `ETFormat` has no member for BC7, so the descriptor keeps whatever it said and the panel states that an SDK
  // rebuild would return the texture to the named format.
  let root: PathBuf = editing_directory("bc7");
  let texture_path: PathBuf = root.join("ston_beton05.dds");
  let descriptor_path: PathBuf = root.join("ston_beton05.thm");

  std::fs::write(&texture_path, b"placeholder").expect("texture is writable");
  std::fs::write(&descriptor_path, ThmFixture::image().to_bytes()).expect("descriptor is writable");

  let before: Option<ThmFormat> = read_descriptor(&descriptor_path)
    .texture_param
    .map(|param| param.format);

  TextureSaver::save(
    &AssetMountState::new(),
    &JobHandle::inert(),
    &descriptor_save(
      &descriptor_path,
      TextureDescriptorForm::read(&read_descriptor(&descriptor_path)),
    ),
    Some(texture_write(
      &texture_path,
      TextureEncodingFormat::Bc7,
      encoded(ImageFormat::BC7RgbaUnorm),
    )),
  )
  .expect("save succeeds");

  assert_eq!(
    read_descriptor(&descriptor_path)
      .texture_param
      .map(|param| param.format),
    before
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

  let outcome: TextureSaveOutcome = TextureSaver::save(
    &AssetMountState::new(),
    &job,
    &descriptor_save(&descriptor_path, TextureDescriptorForm::read(&ThmFile::new_texture())),
    Some(texture_write(&texture_path, TextureEncodingFormat::Bc3, vec![0; 128])),
  )
  .expect("a stopped save is a result, not a failure");

  assert_eq!(outcome.outcome, JobOutcome::Cancelled);
  assert!(outcome.written.is_empty());
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
    TextureSaver::save(
      &AssetMountState::new(),
      &job,
      &TextureDescriptorSave {
        target: stamped,
        form: TextureDescriptorForm::read(&ThmFile::new_texture()),
      },
      None,
    )
    .is_err_and(|error| error.contains("changed on disk"))
  );
}
