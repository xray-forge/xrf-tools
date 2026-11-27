use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use xrf_test_utils::GeneratedResources;
use xrf_utils::StagedFile;
use xrf_vfs::{XrayInputRecord, XrayMountMode, XrayRevision, XrayRoots, XraySearch};

use crate::assets::change::{AssetChangeOrder, AssetChangeTicket};
use crate::assets::{AssetMountState, FileStamp};

/// A ticket dropped undelivered, as a fan-out that failed drops its own, gives its turn up: the events issued after it
/// are heard all the same, in their order.
#[test]
fn a_ticket_given_up_lets_later_changes_be_heard() {
  let order: Arc<AssetChangeOrder> = Arc::new(AssetChangeOrder::default());
  let heard: Arc<Mutex<Vec<u32>>> = Arc::new(Mutex::new(Vec::new()));
  let given_up: AssetChangeTicket = order.issue_next();
  let second: AssetChangeTicket = order.issue_next();
  let third: AssetChangeTicket = order.issue_next();

  for (ticket, number) in [(third, 3), (second, 2)] {
    let heard: Arc<Mutex<Vec<u32>>> = Arc::clone(&heard);

    ticket.deliver(move || heard.lock().unwrap().push(number));
  }

  assert!(
    heard.lock().unwrap().is_empty(),
    "nothing is heard before the first turn"
  );

  drop(given_up);

  assert_eq!(*heard.lock().unwrap(), vec![2, 3]);
}

#[test]
fn a_stamp_tells_a_missing_file_from_an_unreadable_one() {
  let root: PathBuf = GeneratedResources::path("core/assets/stamp");
  let path: PathBuf = root.join("ston_beton05.thm");

  let _ = fs::remove_dir_all(&root);

  fs::create_dir_all(&root).expect("case directory");

  assert_eq!(FileStamp::read(&path).expect("absent is not an error"), None);

  fs::write(&path, b"twelve bytes").expect("file is writable");

  assert_eq!(
    FileStamp::read(&path)
      .expect("present is readable")
      .map(|stamp| stamp.size),
    Some(12)
  );
}

/// A tree of `files` holding `payload`, mounted by a new state, with a search over it and its root.
fn mounted_tree(name: &str, files: &[&str]) -> (AssetMountState, PathBuf, XraySearch) {
  let root: PathBuf = GeneratedResources::path(&format!("core/assets/{name}"));

  let _ = fs::remove_dir_all(&root);

  for file in files {
    let path: PathBuf = root.join(file);

    fs::create_dir_all(path.parent().expect("a file sits in a directory")).expect("directory created");
    fs::write(&path, b"payload").expect("file written");
  }

  let assets: AssetMountState = AssetMountState::new();
  let search: XraySearch = assets
    .search(&XrayRoots::one(root.clone(), XrayMountMode::Directory))
    .expect("the tree mounts");

  (assets, root, search)
}

fn stage(target: &Path, contents: &str) -> StagedFile {
  StagedFile::stage(
    target,
    &StagedFile::name_sibling(target, "staged").expect("the target sits in a directory"),
    contents.as_bytes(),
  )
  .expect("the writer stages its file")
}

/// Every file in a directory, staged leftovers included.
fn list_names(directory: &Path) -> Vec<String> {
  let mut names: Vec<String> = fs::read_dir(directory)
    .expect("the directory lists")
    .map(|entry| entry.expect("an entry").file_name().to_string_lossy().into_owned())
    .collect();

  names.sort();

  names
}

#[test]
fn writes_a_mounted_file_as_a_commit_and_one_outside_every_mount_in_place() {
  let (assets, root, _) = mounted_tree("write_files", &["configs/system.ltx"]);
  let outside: PathBuf = root.with_file_name("write_files_outside");
  let before: XrayRevision = assets.get_vfs().get_revision();

  let _ = fs::remove_dir_all(&outside);

  fs::create_dir_all(&outside).expect("the outside folder is made");

  let written: Vec<PathBuf> = assets
    .write_files(
      vec![
        stage(&outside.join("export.ltx"), "exported"),
        stage(&root.join("configs/system.ltx"), "edited"),
      ],
      None,
    )
    .expect("the files are written");

  assert_eq!(
    written,
    [root.join("configs/system.ltx"), outside.join("export.ltx")],
    "the mounted files first"
  );
  assert_ne!(assets.get_vfs().get_revision(), before, "the mounted write is a commit");
  assert_eq!(
    assets
      .search(&XrayRoots::one(root.clone(), XrayMountMode::Directory))
      .expect("the tree mounts")
      .read("configs\\system.ltx")
      .expect("the file reads"),
    b"edited"
  );
  assert_eq!(
    fs::read(outside.join("export.ltx")).expect("the export reads"),
    b"exported"
  );
  assert_eq!(list_names(&root.join("configs")), ["system.ltx"]);
  assert_eq!(list_names(&outside), ["export.ltx"]);
}

#[test]
fn refuses_a_write_whose_reads_moved_and_leaves_nothing_staged() {
  let (assets, root, search) = mounted_tree("write_files_stale", &["configs/system.ltx"]);
  let record: XrayInputRecord = XrayInputRecord::new();
  let target: PathBuf = root.join("configs/system.ltx");

  search
    .with_recording(&record)
    .read("configs\\system.ltx")
    .expect("the writer reads the file");
  assets
    .write_files(vec![stage(&target, "someone else")], None)
    .expect("another writer lands first");

  let error: String = assets
    .write_files(vec![stage(&target, "the writer")], Some(&record))
    .expect_err("what the writer read moved");

  assert!(error.contains("stale"), "{error}");
  assert_eq!(fs::read(&target).expect("the file reads"), b"someone else");
  assert_eq!(list_names(&root.join("configs")), ["system.ltx"]);
}

#[test]
fn a_stamp_is_the_size_and_the_modification_time_together() {
  // Both, because a rewrite that happens to land on the same length is exactly the change a size alone would miss.
  let stamp: FileStamp = FileStamp {
    size: 12,
    modified_ms: 1_700_000_000_000,
  };

  assert_ne!(stamp, FileStamp { size: 13, ..stamp });
  assert_ne!(
    stamp,
    FileStamp {
      modified_ms: 1_700_000_000_001,
      ..stamp
    }
  );
}
