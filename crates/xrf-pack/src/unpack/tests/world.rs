use std::fs;
use std::path::{Path, PathBuf};

use xrf_job::JobOutcome;
use xrf_test_utils::utils::build_absolute_generated_test_resource_path;
use xrf_vfs::{XrayProbe, XrayProbePlan, XrayProbeStep, XrayVfs};

use crate::unpack::archive_extract_result::{ArchiveExtractDirectoryResult, ArchiveExtractResult};
use crate::unpack::xray_world_extractor::XrayWorldExtractor;

fn tree(name: &str, files: &[(&str, &str)]) -> PathBuf {
  let root: PathBuf = build_absolute_generated_test_resource_path(&format!("xray_world_extractor/{name}"));

  let _ = fs::remove_dir_all(&root);

  for (path, contents) in files {
    let path: PathBuf = root.join(path);

    fs::create_dir_all(path.parent().expect("a file sits in a directory")).expect("test tree is creatable");
    fs::write(&path, contents).expect("test file is writable");
  }

  root
}

/// A mod tree in front of the tree it overrides, which is the arrangement extraction has to answer for.
fn layered(name: &str) -> (XrayVfs, Vec<XrayProbeStep>) {
  let front: PathBuf = tree(
    &format!("{name}_front"),
    &[("configs/system.ltx", "front"), ("configs/only_front.ltx", "front")],
  );
  let back: PathBuf = tree(
    &format!("{name}_back"),
    &[
      ("configs/system.ltx", "back"),
      ("configs/only_back.ltx", "back"),
      ("textures/outside.dds", "back"),
    ],
  );

  let mut vfs: XrayVfs = XrayVfs::new();

  let steps: Vec<XrayProbeStep> = XrayProbePlan::new()
    .with_root("front", &front)
    .expect("front plans")
    .with_root("back", &back)
    .expect("back plans")
    .mount_into(&mut vfs)
    .expect("both mount");

  (vfs, steps)
}

fn destination(name: &str) -> PathBuf {
  let root: PathBuf = build_absolute_generated_test_resource_path(&format!("xray_world_extractor/{name}_out"));

  let _ = fs::remove_dir_all(&root);

  root
}

#[test]
fn a_directory_extraction_writes_the_copy_the_world_resolves() {
  let (vfs, steps) = layered("directory");
  let probe: XrayProbe = vfs.probe().with_steps(steps);
  let out: PathBuf = destination("directory");

  let result: ArchiveExtractDirectoryResult =
    XrayWorldExtractor::extract_directory(&probe, "configs", &out).expect("the prefix resolves");

  assert_eq!(result.outcome, JobOutcome::Completed);
  assert_eq!(result.extracted_count, 3, "one file per engine path under the prefix");

  // The prefix itself is not repeated below the destination, as an archive extraction also declines to do.
  assert_eq!(
    fs::read_to_string(out.join("system.ltx")).expect("the winner is written"),
    "front",
    "the shadowed copy is never the one written"
  );
  assert!(out.join("only_front.ltx").is_file());
  assert!(out.join("only_back.ltx").is_file(), "a path only the lower tree holds");
  assert!(
    !out.join("outside.dds").exists() && !out.join("textures").exists(),
    "and nothing outside the prefix"
  );
}

#[test]
fn an_empty_prefix_extracts_the_whole_world_with_its_layout() {
  let (vfs, steps) = layered("whole");
  let probe: XrayProbe = vfs.probe().with_steps(steps);
  let out: PathBuf = destination("whole");

  let result: ArchiveExtractDirectoryResult =
    XrayWorldExtractor::extract_directory(&probe, "", &out).expect("the root resolves");

  assert_eq!(result.extracted_count, 4);
  assert!(out.join("configs").join("system.ltx").is_file());
  assert!(out.join("textures").join("outside.dds").is_file());
}

#[test]
fn a_prefix_nothing_resolves_under_is_refused_rather_than_writing_an_empty_tree() {
  let (vfs, steps) = layered("absent");
  let probe: XrayProbe = vfs.probe().with_steps(steps);

  assert!(XrayWorldExtractor::extract_directory(&probe, "meshes", destination("absent")).is_err());
}

#[test]
fn a_prefix_matches_on_whole_segments() {
  // The trap a raw prefix comparison walks into, and the reason extraction shares the packer's boundary rule.
  let root: PathBuf = tree(
    "segments",
    &[("configs/a.ltx", "keep"), ("configs_backup/b.ltx", "leave")],
  );

  let mut vfs: XrayVfs = XrayVfs::new();
  let steps: Vec<XrayProbeStep> = XrayProbePlan::new()
    .with_root("root", &root)
    .expect("plans")
    .mount_into(&mut vfs)
    .expect("mounts");

  let probe: XrayProbe = vfs.probe().with_steps(steps);
  let out: PathBuf = destination("segments");

  let result: ArchiveExtractDirectoryResult =
    XrayWorldExtractor::extract_directory(&probe, "configs", &out).expect("the prefix resolves");

  assert_eq!(result.extracted_count, 1);
  assert!(out.join("a.ltx").is_file());
  assert!(!out.join("b.ltx").exists());
}

#[test]
fn one_file_is_written_where_the_caller_pointed() {
  let (vfs, steps) = layered("file");
  let probe: XrayProbe = vfs.probe().with_steps(steps);
  let out: PathBuf = destination("file");
  let target: PathBuf = out.join("nested").join("picked.ltx");

  let result: ArchiveExtractResult =
    XrayWorldExtractor::extract_file(&probe, "configs\\system.ltx", &target).expect("the path resolves");

  assert_eq!(result.name, "configs\\system.ltx");
  assert_eq!(result.size, "front".len() as u64);
  assert_eq!(
    fs::read_to_string(&target).expect("the file is written"),
    "front",
    "a single extraction resolves the same copy a browser previews"
  );
}

#[test]
fn a_file_nothing_resolves_is_refused() {
  let (vfs, steps) = layered("file_absent");
  let probe: XrayProbe = vfs.probe().with_steps(steps);

  assert!(
    XrayWorldExtractor::extract_file(
      &probe,
      "configs\\absent.ltx",
      destination("file_absent").join("absent.ltx")
    )
    .is_err()
  );
}

#[test]
fn an_entry_name_cannot_write_outside_the_destination() {
  // Names come from the mounts rather than from the caller, so containment is the writer's property: proven here
  // through the destination the extractor writes every entry through.
  let (vfs, steps) = layered("contained");
  let probe: XrayProbe = vfs.probe().with_steps(steps);
  let out: PathBuf = destination("contained");

  XrayWorldExtractor::extract_directory(&probe, "", &out).expect("the root resolves");

  for entry in walk(&out) {
    assert!(
      entry.starts_with(&out),
      "'{}' was written outside the destination",
      entry.display()
    );
  }
}

fn walk(root: &Path) -> Vec<PathBuf> {
  let mut found: Vec<PathBuf> = Vec::new();

  for entry in fs::read_dir(root).expect("the destination exists").flatten() {
    let path: PathBuf = entry.path();

    if path.is_dir() {
      found.extend(walk(&path));
    } else {
      found.push(path);
    }
  }

  found
}
