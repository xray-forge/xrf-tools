# xrf-omf

Reads, edits and writes `.omf` motion libraries, the shared animation banks a skinned visual references by name: a
weapon's hand animations, a stalker's walk cycles. `OmfFile` holds one bank as the two chunks
[xrf-skeleton](../xrf-skeleton/README.md) owns, which a self-animated OGF also embeds. The CLI's `omf` commands, the
weapon, HUD animation and mesh verifiers, and the desktop app's model viewer and archive describer read banks through
it.

## Read, edit and write a bank

```rust
use xrf_chunk::XRayByteOrder;
use xrf_omf::{OmfFile, OmfMotionsProcessor};
use xrf_skeleton::{
  SkeletonMotion, SkeletonMotionDefinition, SkeletonMotionParametersChunk, SkeletonMotionsChunk, SkeletonPart,
};

fn main() -> xrf_error::XrfResult {
  let mut bank = OmfFile {
    parameters: SkeletonMotionParametersChunk {
      version: 4,
      parts: vec![SkeletonPart {
        name: "default".into(),
        bones: vec![("bip01".into(), 0)],
      }],
      motions: vec![SkeletonMotionDefinition {
        name: "pm_idle".into(),
        flags: 0,
        bone_or_part: 0,
        motion: 0,
        speed: 1.0,
        power: 1.0,
        accrue: 2.0,
        falloff: 2.0,
        marks: Vec::new(),
      }],
    },
    motions: SkeletonMotionsChunk {
      motions: vec![SkeletonMotion {
        label: "pm_idle".into(),
        count: 1,
        flags: 0,
        remaining: vec![0; 4],
      }],
    },
  };

  OmfMotionsProcessor::duplicate_motion(&mut bank, "pm_idle", "pm_idle_bore", true)?;

  let mut bytes: Vec<u8> = Vec::new();
  bank.write_to::<XRayByteOrder>(&mut bytes)?;

  let restored = OmfFile::read_from_bytes::<XRayByteOrder>(bytes)?;

  assert_eq!(restored.get_motion_names(), ["pm_idle", "pm_idle_bore"]);
  assert!(
    restored
      .get_motion_by_name("pm_idle_bore")
      .is_some_and(|(definition, motion)| {
        let plays_once: bool = definition.flags & OmfMotionsProcessor::FLAG_PLAY_ONCE != 0;

        definition.motion == 1 && plays_once && motion.label == "pm_idle_bore"
      })
  );

  Ok(())
}
```

Read a bank with `read_from_path`, `read_from_file`, `read_from_bytes` for an archived entry, `read_from_chunk`, or
`read_from_chunks` over children already read. Write one with `write_to_path`, which creates missing parent
directories, or `write_to` for any writer. A bank is exactly two top-level chunks: the motion payloads
(`SkeletonMotionsChunk`, ID 14) and the parameters (`SkeletonMotionParametersChunk`, ID 15). The writer emits them in
that order; the reader finds them by ID.

## Motion identity

A motion is its definition paired with the key payload at the same ordinal, as the engine resolves it. Use
`get_motions`, `get_motion_by_name` and `get_motion_names`, which read names from the definitions. The payload's
`label` and the definition's `motion` field are kept as the file stores them but are not identities: third-party banks
carry labels that no longer name their motion and `motion` values that address nothing, and both survive a rewrite.
`get_diverging_labels_count` counts payloads whose label no longer matches its motion's name, ignoring case.
`get_bones` and `get_bones_count` list the bones the partitions name.

## Edit motions

`OmfMotionsProcessor` edits the motion set while keeping definitions and payloads paired:

- `retain_motions` keeps the motions whose name a predicate accepts and returns how many it removed.
- `rename_motions` renames by an old-to-new map and returns how many it renamed.
- `duplicate_motion` copies a motion under a new name, optionally setting `FLAG_PLAY_ONCE` (`esmStopAtEnd`) so the copy
  stops at its end rather than looping.

A payload label follows a rename or copy only where it matched the old name; a label that never named its motion stays
as stored. Where every `motion` field equals its ordinal, filtering and copying renumber them to stay ordinal;
otherwise they are left untouched. Every operation fails when definitions and payloads are unpaired, and renaming and
copying fail when a name they introduce would share another motion's name ignoring case: the engine maps motions by
their lowercased names and keeps the first, so the later would be unreachable. A collision the bank already holds does
not block an edit, and a refused edit changes nothing.

## Errors and checks

Reads reject a file that cannot be opened, any number of top-level chunks other than two, a missing chunk, a parameters
version newer than 4, and unequal definition and payload counts. As the engine does, they accept every older version,
reading marks from version 4 on, and find each motion payload by its chunk id, one more than its index, rather than by
position. A bank holding 16383 motions or more is refused, since a motion id addresses fewer. Writes reject unequal
counts, those motion counts, and versions newer than 4.
Two shipped Anomaly HUD banks are truncated and refuse to read.

Run `cargo test --locked -p xrf-omf` from the repository root. See the [bank](src/omf_file.rs) and the
[motion editor](src/omf_motions_processor.rs).
