//! Pins how an entry is routed to a describer: the ceiling a held read answers to, a claim that does not hold falling
//! to the container walk, and a lookup that finds an entry however a reference spells it.

use std::fs;
use std::path::PathBuf;

use xrf_test_utils::utils::build_absolute_generated_test_resource_path;
use xrf_vfs::{XrayMountMode, XrayRoot, XrayRoots, XraySearch, XraySearchStep, XrayVfs};

use crate::browse::ArchiveWorld;
use crate::describe::archive_file_description::{ArchiveDescribeRefusal, ArchiveFormatDescription};
use crate::describe::{ArchiveDescribeSource, ArchiveFileDescription};

/// A world over a tree of its own holding `files`, emptied first so a re-run reads what this run wrote.
fn world(case: &str, files: &[(&str, &[u8])]) -> ArchiveWorld {
  let root: PathBuf = build_absolute_generated_test_resource_path(&format!("archives/describe/{case}"));

  let _ = fs::remove_dir_all(&root);

  for (path, contents) in files {
    let path: PathBuf = root.join(path);

    fs::create_dir_all(path.parent().expect("a file sits in a directory")).expect("test tree is creatable");
    fs::write(&path, contents).expect("test file is writable");
  }

  let roots: XrayRoots = XrayRoots::new([XrayRoot::new(root, XrayMountMode::Directory)]);
  let mut vfs: XrayVfs = XrayVfs::new();
  let steps: Vec<XraySearchStep> = roots
    .to_search_plan()
    .expect("the roots plan")
    .mount_into(&mut vfs)
    .expect("the roots mount");
  let search: XraySearch = XraySearch::new(vfs).with_steps(steps);

  ArchiveWorld::list(&search, search.clone(), roots)
}

fn describe(world: &ArchiveWorld, name: &str) -> ArchiveFormatDescription {
  ArchiveFileDescription::of(&ArchiveDescribeSource::World { world }, name)
    .expect("the entry is described")
    .format
}

#[test]
fn a_held_read_past_the_ceiling_is_refused_rather_than_read() {
  let mut world: ArchiveWorld = world("too_large", &[("textures\\wall.thm", &[0; 64])]);

  world.read_policy.maximum_describe_size = 16;

  assert_eq!(
    describe(&world, "textures\\wall.thm"),
    ArchiveFormatDescription::Unsupported {
      reason: ArchiveDescribeRefusal::TooLarge { size: 64, maximum: 16 }
    }
  );
}

#[test]
fn a_spawn_that_is_neither_format_falls_to_the_container_walk() {
  let world: ArchiveWorld = world(
    "spawn_neither",
    &[("spawns\\broken.spawn", b"not a spawn of either kind")],
  );

  assert!(
    matches!(
      describe(&world, "spawns\\broken.spawn"),
      ArchiveFormatDescription::Unsupported {
        reason: ArchiveDescribeRefusal::NoDescriber { .. }
      } | ArchiveFormatDescription::Chunks { .. }
    ),
    "a claim that does not hold is offered whatever the container walk can say"
  );
}

#[test]
fn a_spawn_read_whole_as_a_level_list_answers_to_the_ceiling() {
  // A set is read by seeking and answers to no ceiling, so a damaged one past it must not be held and parsed whole as
  // a level's object list instead.
  let mut world: ArchiveWorld = world("spawn_oversized", &[("spawns\\all.spawn", &[0; 64])]);

  world.read_policy.maximum_describe_size = 16;

  assert_eq!(
    describe(&world, "spawns\\all.spawn"),
    ArchiveFormatDescription::Unsupported {
      reason: ArchiveDescribeRefusal::TooLarge { size: 64, maximum: 16 }
    }
  );
}

#[test]
fn a_world_lookup_finds_an_entry_however_a_reference_spells_it() {
  let world: ArchiveWorld = world("spelling", &[("textures\\act\\act_arm_1.dds", b"dds")]);
  let source: ArchiveDescribeSource = ArchiveDescribeSource::World { world: &world };

  assert_eq!(
    source.find_entry("Textures/Act/ACT_arm_1.DDS").as_deref(),
    Some("textures\\act\\act_arm_1.dds")
  );
  assert_eq!(source.get_size_of("textures\\act\\act_arm_1.dds"), Some(3));
  assert_eq!(source.find_entry("textures\\act\\act_arm_2.dds"), None);
}
