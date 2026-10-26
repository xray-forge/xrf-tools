//! Pins how an entry is routed to a describer: the ceiling a held read answers to, a claim that does not hold falling
//! to the container walk, and a lookup and a read that find an entry however a reference spells it.

use xrf_archive::{ARCHIVE_READ_POLICY, ArchiveProject, ArchiveReadPolicy};
use xrf_vfs::{XrayMountMode, XrayRoots};

use crate::browse::{ArchiveNameIndex, ArchiveWorld};
use crate::describe::description::{ArchiveDescribeRefusal, ArchiveFormatDescription};
use crate::describe::{ArchiveDescribeSource, ArchiveFileDescription};
use crate::tests::fixtures;

/// One chunk holding four bytes: a container, but no format any describer claims.
const CONTAINER: &[u8] = &[7, 0, 0, 0, 4, 0, 0, 0, 1, 2, 3, 4];

/// A world over a tree of its own holding `files`.
fn world(case: &str, files: &[(&str, &[u8])]) -> ArchiveWorld {
  fixtures::world(&XrayRoots::one(
    fixtures::tree(&format!("describe/{case}"), files),
    XrayMountMode::Directory,
  ))
}

fn describe(world: &ArchiveWorld, name: &str) -> ArchiveFormatDescription {
  describe_under(world, name, &ARCHIVE_READ_POLICY)
}

fn describe_under(world: &ArchiveWorld, name: &str, policy: &ArchiveReadPolicy) -> ArchiveFormatDescription {
  ArchiveFileDescription::of(&ArchiveDescribeSource::World { world }, name, policy)
    .expect("the entry is described")
    .format
}

/// The viewers' policy, with a describe ceiling small enough for a test entry to pass.
const SMALL_CEILING: ArchiveReadPolicy = ArchiveReadPolicy {
  maximum_describe_size: 16,
  ..ARCHIVE_READ_POLICY
};

#[test]
fn a_held_read_past_the_ceiling_is_refused_rather_than_read() {
  let world: ArchiveWorld = world("too_large", &[("textures\\wall.thm", &[0; 64])]);

  assert_eq!(
    describe_under(&world, "textures\\wall.thm", &SMALL_CEILING),
    ArchiveFormatDescription::Unsupported {
      reason: ArchiveDescribeRefusal::TooLarge { size: 64, maximum: 16 }
    }
  );
}

#[test]
fn a_spawn_that_is_neither_format_falls_to_the_container_walk() {
  // A container with no set header and no spawn packet in it.
  let world: ArchiveWorld = world(
    "spawn_neither",
    &[
      ("spawns\\container.spawn", CONTAINER),
      ("spawns\\broken.spawn", b"not a spawn of either kind"),
    ],
  );

  let ArchiveFormatDescription::Chunks { description } = describe(&world, "spawns\\container.spawn") else {
    panic!("a claim that does not hold is offered whatever the container walk can say");
  };

  assert_eq!((description.nodes, description.size), (1, CONTAINER.len() as u64));
  assert_eq!(
    describe(&world, "spawns\\broken.spawn"),
    ArchiveFormatDescription::Unsupported {
      reason: ArchiveDescribeRefusal::NoDescriber {
        extension: String::from("spawn")
      }
    },
    "and bytes that are no container at all say nothing describes them"
  );
}

#[test]
fn a_spawn_read_whole_as_a_level_list_answers_to_the_ceiling() {
  // A set is read by seeking and answers to no ceiling, so a damaged one past it must not be held and parsed whole as
  // a level's object list instead.
  let world: ArchiveWorld = world("spawn_oversized", &[("spawns\\all.spawn", &[0; 64])]);

  assert_eq!(
    describe_under(&world, "spawns\\all.spawn", &SMALL_CEILING),
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

#[test]
fn a_volume_set_describes_an_entry_however_a_reference_spells_it() {
  let project: ArchiveProject = ArchiveProject::new(fixtures::volumes(
    "describe/volumes_spelling",
    &[("meshes/container.dat", CONTAINER)],
  ))
  .expect("the volumes open");
  let names: ArchiveNameIndex = ArchiveNameIndex::default();
  let source: ArchiveDescribeSource = ArchiveDescribeSource::Volumes {
    project: &project,
    names: &names,
  };

  // The lookup that weighs the entry and the read that walks it must name the same file, as they do in a world.
  let ArchiveFormatDescription::Chunks { description } =
    ArchiveFileDescription::of(&source, "Meshes/Container.DAT", &ARCHIVE_READ_POLICY)
      .expect("the entry is described")
      .format
  else {
    panic!("an unclaimed container is walked");
  };

  assert_eq!(description.size, CONTAINER.len() as u64);
  assert_eq!(
    source.read_bytes("MESHES\\container.dat").expect("the entry reads"),
    CONTAINER
  );
}
