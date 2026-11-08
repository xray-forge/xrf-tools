# xrf-editable-level

Reads and writes the Level Editor's scenes, the source a compiled level is built from: the `.level` main file under
`$maps$` and, in the directory beside it named after it, one part file per tool. "Editable level" is this crate's
name for what the editor calls its scene (`EScene`), after [xrf-editable-object](../xrf-editable-object/README.md),
which holds the `.object` files a scene places.

## Read a scene

```rust,no_run
use xrf_chunk::XRayByteOrder;
use xrf_editable_level::{EditableLevelPart, EditableLevelScene};

fn main() -> xrf_error::XrfResult {
  let scene = EditableLevelScene::read_from_path::<XRayByteOrder, _>(&"rawdata/levels/labx8.level")?;

  println!("builds to {}", scene.level.get_level_options()?.level_path);

  for tool in &scene.tools {
    for part in &tool.parts {
      if let EditableLevelPart::Text(part) = part {
        println!("{}: {} objects", tool.name, part.get_objects()?.len());
      }
    }
  }

  Ok(())
}
```

## What is read

| Type                 | File                                   | Holds                                                                 |
| -------------------- | -------------------------------------- | --------------------------------------------------------------------- |
| `EditableLevelFile`  | `<name>.level`                         | Version, GUID, level options, build parameters, camera, owner tag     |
| `EditableTextPart`   | `<name>\<tool>.part`, saved as text    | GUID, `[main]`, `[modif]`, `[object_N]` and the sections named after it |
| `EditableBinaryPart` | `<name>\<tool>.part`, saved as chunks  | GUID chunk `0x7000`, the tool's stream in chunk `0x8000` plus its class |
| `EditableLevelScene` | the `.level` and its part directory    | The main file and each tool's parts                                   |

The `.level` and the text parts are text as `CInifile::save_as` writes it, read and written through
[xrf-ini](../xrf-ini/README.md)'s `IniFormat::XRAY_SAVE_AS`, so a file the editor saved comes back byte for byte. Each
keeps the whole `IniFile` as its `ini` field, and typed getters and setters read and write its sections in the editor's
layout: `EditableLevelOptions`, `EditableBuildParams`, `EditablePartHeader`, and each object's class, name and transform
through `EditablePartObject`. A spawn's data is its `[object_N_spawndata]` section: one numbered field per value its
packet holds, as the editor's ini stream writes them, multi-line custom data included. `EditableLevelValue` formats
values as the editor does: `%f` floats and `x, y, z` vectors. Not every number a saved scene holds is an `f32` printed
that way (`labx8.level`'s camera stands at `-80.052984`, which no `f32` prints as), so typed values are for use and a
file writes back exactly through the text it keeps.

The AI map, detail objects and wall marks save as chunks. A binary part keeps the tool's stream as its bytes, and its
chunks' stored order and any chunk the editor never writes in its `ChunkLayout`, so it too comes back byte for byte.

A scene is read as the editor reads it: a part file opening with `[` is text and any other is chunks, a tool's
numbered files (`group.part1`, `group.part2`) follow its first until one is missing, and files of other names, such as
the editor's `.~part` copies of an earlier save, are no part of the scene.

## Typed objects

An object tool's part reads typed as an `EditableObjectPart<T>`: the GUID, `[main]`'s version and, for the tools that
keep them, flags, `[modif]`, and the objects `[object_N]` numbers, each an `EditableToolObject` that reads its sections
and writes them as the editor's `SaveLTX` does. A part refuses an object of another tool's class.

| Type                      | Editor                   | Holds                                                              |
| ------------------------- | ------------------------ | ------------------------------------------------------------------ |
| `EditableCustomObject`    | `CCustomObject`          | Every object's flags, name and transform                           |
| `EditableObjectTransform` | `CCustomObject`          | Position, rotation angles and scale, and the matrix they compose   |
| `EditableParticleObject`  | `EParticlesObject`       | An effect the scene plants: its `particles.xr` name and game modes |

`EditableObjectTransform::to_matrix` composes as `CCustomObject::OnUpdateTransform` does, scaled, then turned by
`setXYZi` of the negated angles, then moved, through `Fmatrix::mul` in the editor's order of sums, so it gives the bits
the editor exported. `from_matrix` decomposes one, and `round_as_saved` rounds a transform as its `%f` text reads back,
which is what the editor composed from. The editor's own `%f` is not exact: it prints some floats a unit off in the last
digit (`-25.461156` where the float is nearer `-25.461157`), so a typed write differs from its file there and reads back
the same.

## Errors and checks

Reads refuse what the editor refuses or skips: a `.level` of a version other than 5, a scene or text part with no GUID,
a binary part with no GUID chunk, a GUID chunk of other than 16 bytes, or no tool data chunk, and a part whose GUID is
not its scene's. `EditableTextPart::get_objects` refuses an object `[main]` counts but no section holds. Writes refuse
what [xrf-ini](../xrf-ini/README.md) refuses, text that would not read back as itself, and a scene refuses to write,
before writing anything, into a part directory holding a part file it would not write: a tool it no longer has or a
numbered file past its last, which the editor would read as part of the scene.

Run `cargo test --locked -p xrf-editable-level` from the repository root. See the
[scene](src/scene/editable_level_scene.rs), the [parts](src/part/) and the [tests](src/tests/).
