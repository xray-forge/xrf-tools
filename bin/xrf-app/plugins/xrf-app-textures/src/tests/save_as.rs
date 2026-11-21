use std::fs;
use std::path::{Path, PathBuf};

use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetMountState;
use xrf_job::{JobHandle, JobOutcome};
use xrf_material::fixtures::FixtureTree;
use xrf_pack::{ArchivePackConfig, ArchivePackDirectory, ArchivePacker};
use xrf_test_utils::GeneratedResources;
use xrf_thm::ThmFile;
use xrf_vfs::{XrayMountMode, XrayRoot, XrayRoots};

use crate::edit::{
  TextureDescriptorForm, TextureEditTargets, TextureReferenceField, TextureSaveAsBump, TextureSaveAsOutcome,
  TextureSaveAsPlan, TextureSaveAsRole, TextureSaveAsTarget, TextureSaveAsWriter, TexturesSaveAsRequest,
};
use crate::tests::fixtures::{BASE, BUMP, bumped_tree, read_descriptor, roots_of};

const COPY: &str = "ston\\ston_beton05_copy";

fn sources(tree: &FixtureTree) -> TextureEditTargets {
  let directory: PathBuf = tree.root().join("textures").join("ston");

  TextureEditTargets::of_paths(&directory.join("ston_beton05.thm"), &directory.join("ston_beton05.dds"))
    .expect("targets are readable")
}

fn target(roots: XrayRoots, destination: &str) -> TextureSaveAsTarget {
  TextureSaveAsTarget {
    roots,
    reference: String::from(BASE),
    destination: String::from(destination),
  }
}

fn request(tree: &FixtureTree, destination: &str) -> TexturesSaveAsRequest {
  let sources: TextureEditTargets = sources(tree);

  TexturesSaveAsRequest {
    target: target(roots_of(tree), destination),
    form: TextureDescriptorForm::read(&read_descriptor(Path::new(&sources.descriptor.path))),
    sources,
    texture: None,
    bump: TextureSaveAsBump::default(),
    is_forced: false,
  }
}

fn write(assets: &AssetMountState, request: &TexturesSaveAsRequest) -> TauriResult<TextureSaveAsOutcome> {
  TextureSaveAsWriter::write(assets, &JobHandle::inert(), request, None)
}

fn plan(tree: &FixtureTree, destination: &str, form: &TextureDescriptorForm) -> TauriResult<TextureSaveAsPlan> {
  let roots: XrayRoots = roots_of(tree);

  TextureSaveAsPlan::build(
    &AssetMountState::new().search(&roots).expect("the tree mounts"),
    &target(roots, destination),
    form,
    TextureSaveAsBump::default(),
  )
}

fn copied(tree: &FixtureTree, name: &str) -> PathBuf {
  tree.root().join("textures").join("ston").join(name)
}

#[test]
fn a_copy_writes_the_texture_its_pair_and_its_descriptor_under_the_new_name() {
  let tree: FixtureTree = bumped_tree("save_as_whole");
  let assets: AssetMountState = AssetMountState::new();
  let original: Vec<u8> = fs::read(copied(&tree, "ston_beton05.thm")).expect("descriptor is readable");

  let outcome: TextureSaveAsOutcome = write(&assets, &request(&tree, COPY)).expect("the copy is written");

  assert_eq!(outcome.outcome, JobOutcome::Completed);
  assert_eq!(outcome.reference, COPY);

  // The texture first and its descriptor after it, then the pair: the order the plan names them in.
  for (written, name) in outcome.written.iter().zip([
    "ston_beton05_copy.dds",
    "ston_beton05_copy.thm",
    "ston_beton05_copy_bump.dds",
    "ston_beton05_copy_bump#.dds",
  ]) {
    assert!(written.ends_with(name), "expect {name}, got {written}");
    assert!(Path::new(written).is_file());
  }

  assert_eq!(outcome.written.len(), 4);
  assert_eq!(
    fs::read(copied(&tree, "ston_beton05_copy_bump.dds")).expect("copied bump is readable"),
    fs::read(copied(&tree, "ston_beton05_bump.dds")).expect("original bump is readable"),
    "expect the pair copied byte for byte, since it is measured against the original's encoder"
  );
  assert_eq!(
    fs::read(copied(&tree, "ston_beton05.thm")).expect("descriptor is readable"),
    original,
    "expect the original left exactly as it was"
  );
}

#[test]
fn a_copy_uses_its_own_bump_map_by_default_and_names_every_field_it_changes() {
  let tree: FixtureTree = bumped_tree("save_as_own");
  let mut asked: TexturesSaveAsRequest = request(&tree, COPY);

  asked.form.detail_name = String::from(BASE);

  let planned: TextureSaveAsPlan = plan(&tree, COPY, &asked.form).expect("the copy plans");

  assert!(planned.has_bump_pair);
  assert_eq!(
    planned
      .renames
      .iter()
      .map(|rename| (rename.field, rename.from.as_str(), rename.to.as_str()))
      .collect::<Vec<_>>(),
    vec![
      (TextureReferenceField::Bump, BUMP, "ston\\ston_beton05_copy_bump"),
      (TextureReferenceField::Detail, BASE, COPY),
    ]
  );

  write(&AssetMountState::new(), &asked).expect("the copy is written");

  let written: ThmFile = read_descriptor(&copied(&tree, "ston_beton05_copy.thm"));

  assert_eq!(
    written.bump.as_ref().map(|bump| bump.name.as_str()),
    Some("ston\\ston_beton05_copy_bump")
  );
  assert_eq!(written.detail.as_ref().map(|detail| detail.name.as_str()), Some(COPY));
}

#[test]
fn a_copy_sharing_the_bump_map_keeps_its_names_and_copies_no_bump_files() {
  let tree: FixtureTree = bumped_tree("save_as_shared");
  let mut asked: TexturesSaveAsRequest = request(&tree, COPY);

  asked.form.detail_name = String::from(BASE);
  asked.bump = TextureSaveAsBump::Shared;

  let outcome: TextureSaveAsOutcome = write(&AssetMountState::new(), &asked).expect("the copy is written");

  // Bump map files nobody would read are not written.
  assert_eq!(outcome.written.len(), 2);
  assert!(!copied(&tree, "ston_beton05_copy_bump.dds").exists());
  assert!(!copied(&tree, "ston_beton05_copy_bump#.dds").exists());

  let written: ThmFile = read_descriptor(&copied(&tree, "ston_beton05_copy.thm"));

  assert_eq!(written.bump.as_ref().map(|bump| bump.name.as_str()), Some(BUMP));
  assert_eq!(written.detail.as_ref().map(|detail| detail.name.as_str()), Some(BASE));
}

#[test]
fn a_name_that_is_not_a_reference_in_the_roots_is_refused() {
  let tree: FixtureTree = bumped_tree("save_as_outside");
  let form: TextureDescriptorForm = request(&tree, COPY).form;

  // A free path on disk, or one climbing out of the roots, names nothing the roots hold.
  for destination in ["..\\..\\escape", "c:\\escape\\wall", ""] {
    assert!(
      plan(&tree, destination, &form).is_err_and(|error| error.contains("not a valid texture name")),
      "expect '{destination}' to be refused"
    );
  }

  assert!(
    plan(&tree, "STON\\Ston_Beton05.dds", &form).is_err_and(|error| error.contains("another name")),
    "expect the original's own name, however it is spelled, to be refused as a save"
  );
  assert!(
    !copied(&tree, "escape").exists(),
    "expect a refused copy to write nothing"
  );
}

#[test]
fn a_texture_served_out_of_an_archive_is_refused() {
  let gamedata: PathBuf = GeneratedResources::path("textures_save_as/archived/gamedata");
  let volumes: PathBuf = GeneratedResources::path("textures_save_as/archived/db");

  let _ = fs::remove_dir_all(GeneratedResources::path("textures_save_as/archived"));

  fs::create_dir_all(gamedata.join("textures").join("ston")).expect("tree is creatable");
  fs::write(gamedata.join("textures").join("ston").join("ston_beton05.dds"), b"dds").expect("texture is writable");

  let mut config: ArchivePackConfig = ArchivePackConfig::new(&gamedata, &volumes, "gamedata");

  config.include_directories = vec![ArchivePackDirectory {
    is_recursive: true,
    path: String::new(),
  }];
  config.is_with_skip_list = false;

  ArchivePacker::pack(&config).expect("the tree packs");

  let roots: XrayRoots = XrayRoots::new([XrayRoot::new(volumes, XrayMountMode::Volumes)]);

  assert!(
    TextureSaveAsPlan::build(
      &AssetMountState::new().search(&roots).expect("the volumes mount"),
      &target(roots, COPY),
      &TextureDescriptorForm::read(&ThmFile::new_texture()),
      TextureSaveAsBump::default(),
    )
    .is_err_and(|error| error.contains("served out of an archive"))
  );
}

#[test]
fn a_copy_that_cannot_land_whole_writes_none_of_its_files() {
  let tree: FixtureTree = bumped_tree("save_as_atomic");
  let assets: AssetMountState = AssetMountState::new();

  // Mounted first, so the snapshot is taken before the stray file below exists.
  assets.search(&roots_of(&tree)).expect("the tree mounts");

  // Somebody else's descriptor appears where the copy's would go, after the roots were read.
  fs::write(copied(&tree, "ston_beton05_copy.thm"), b"theirs").expect("stray file is writable");

  assert!(
    write(&assets, &request(&tree, COPY)).is_err_and(|error| error.contains("appeared on disk")),
    "expect the commit to refuse replacing a file nobody read"
  );

  for name in [
    "ston_beton05_copy.dds",
    "ston_beton05_copy_bump.dds",
    "ston_beton05_copy_bump#.dds",
  ] {
    assert!(!copied(&tree, name).exists(), "expect {name} left unwritten");
  }

  assert_eq!(
    fs::read(copied(&tree, "ston_beton05_copy.thm")).expect("stray file is readable"),
    b"theirs"
  );
}

#[test]
fn a_copy_over_an_existing_one_is_refused_until_forced() {
  let tree: FixtureTree = bumped_tree("save_as_replace");
  let assets: AssetMountState = AssetMountState::new();

  write(&assets, &request(&tree, COPY)).expect("the first copy is written");

  let planned: TextureSaveAsPlan = plan(&tree, COPY, &request(&tree, COPY).form).expect("the copy plans");

  assert!(planned.files.iter().all(|file| file.is_replacing));

  assert!(
    write(&assets, &request(&tree, COPY)).is_err_and(|error| error.contains("force it to replace them")),
    "expect the refusal every replace prompt asks about"
  );

  let mut forced: TexturesSaveAsRequest = request(&tree, COPY);

  forced.is_forced = true;

  assert_eq!(
    write(&assets, &forced).expect("a forced copy replaces").outcome,
    JobOutcome::Completed
  );
}

#[test]
fn a_copy_of_a_file_the_roots_do_not_answer_with_is_refused() {
  // A file opened from outside every root is named by its stem, which the roots answer for with their own file.
  let tree: FixtureTree = bumped_tree("save_as_elsewhere");
  let elsewhere: PathBuf = GeneratedResources::path("textures_save_as/elsewhere/ston_beton05.dds");

  fs::create_dir_all(elsewhere.parent().expect("a file sits in a directory")).expect("directory is creatable");
  fs::write(&elsewhere, b"another texture").expect("texture is writable");

  let mut asked: TexturesSaveAsRequest = request(&tree, COPY);

  asked.sources = TextureEditTargets::of_paths(&elsewhere.with_extension("thm"), &elsewhere).expect("targets");

  assert!(
    write(&AssetMountState::new(), &asked).is_err_and(|error| error.contains("different file")),
    "expect the roots' own texture not to be copied in place of the one read"
  );
  assert!(!copied(&tree, "ston_beton05_copy.dds").exists());
}

#[test]
fn a_texture_without_a_descriptor_or_pair_is_copied_with_one_authored() {
  let tree: FixtureTree = FixtureTree::new("textures_save_as_bare").with_texture(BASE);
  let roots: XrayRoots = roots_of(&tree);
  let planned: TextureSaveAsPlan = TextureSaveAsPlan::build(
    &AssetMountState::new().search(&roots).expect("the tree mounts"),
    &target(roots, COPY),
    &TextureDescriptorForm::read(&ThmFile::new_texture()),
    TextureSaveAsBump::default(),
  )
  .expect("the copy plans");

  assert!(!planned.has_bump_pair);

  assert_eq!(
    planned
      .files
      .iter()
      .map(|file| (file.role, file.from.is_some()))
      .collect::<Vec<_>>(),
    vec![
      (TextureSaveAsRole::Texture, true),
      (TextureSaveAsRole::Descriptor, false),
    ],
    "expect a pair the original does not have left out rather than invented"
  );
}
