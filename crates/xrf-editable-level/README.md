# xrf-editable-level

Reads and writes the Level Editor's scenes, the source a compiled level is built from.

A scene is a `.level` main file under `$maps$` plus a directory beside it, named after it, with one part file per
editor tool. The editor calls it `EScene`; this crate says "editable level", after
[xrf-editable-object](../xrf-editable-object/README.md), which reads the `.object` files a scene places.

Saved files come back byte for byte.

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

## Files and types

| Type                 | File                                  | Holds                                                                   |
| -------------------- | ------------------------------------- | ----------------------------------------------------------------------- |
| `EditableLevelFile`  | `<name>.level`                        | Version, GUID, level options, build parameters, camera, owner tag       |
| `EditableTextPart`   | `<name>\<tool>.part`, saved as text   | GUID, `[main]`, `[modif]`, `[object_N]` and the sections named after it |
| `EditableBinaryPart` | `<name>\<tool>.part`, saved as chunks | GUID chunk `0x7000`, the tool's stream in chunk `0x8000`, and its class |
| `EditableLevelScene` | The `.level` and its part directory   | The main file and each tool's parts                                     |

A scene is found as the editor finds it:

- A part file starting with `[` is text; any other is chunks.
- A tool's numbered files (`group.part1`, `group.part2`) follow its first until one is missing.
- Files with other names, such as the editor's `.~part` backups, are not part of the scene.

### Text files

The `.level` and text parts are written as `CInifile::save_as` writes them, read and written through
[xrf-ini](../xrf-ini/README.md)'s `IniFormat::XRAY_SAVE_AS`.

- Each keeps the whole `IniFile` as its `ini` field. Typed getters and setters work on top of it:
  `EditableLevelOptions`, `EditableBuildParams`, `EditableLevelCamera`, `EditablePartHeader`, and `EditablePartObject`
  for each object's class, name and transform.
- A spawn's data is its `[object_N_spawndata]` section: one numbered field per value its packet holds, multi-line
  custom data included.
- `EditableLevelValue` formats values as the editor does: `%f` floats and `x, y, z` vectors.

Typed values are for use; the kept text is what writes back. Not every saved number is an `f32` printed with `%f`:
`labx8.level`'s camera stands at `-80.052984`, which no `f32` prints as.

### Binary parts

The AI map, detail objects and wall marks are saved as chunks. A binary part keeps the tool's stream as bytes, and its
chunk order and any chunk the editor never writes in a `ChunkLayout`.

A tool's stream reads typed where its tool is typed. `EditableWallmarkTool` (`ESceneWallmarkTool`) holds the wall mark
brush and its decals in slots, read as `LoadStream` reads versions 3 and 4 and written as `SaveStream` writes version
4; each `EditableWallmark` keeps its fan's lit vertices, box and sphere (`get_bounds` gives the ones `AddWallmark`
does). An empty slot stays as stored; the editor drops it on load.

`EditableDetailObjectTool` (`EDetailManager`) holds the detail objects, each an `EditableDetailObject` (`EDetail`: the
library object it plants by reference, its scale range, density factor and flags), the base texture's colour indices,
and the slot grid as stored (`DetailSlot`, four words a slot), with the grid's header, box, base texture, density and
snap objects; `get_slot_object`, `with_slot_object` and `get_unlit_slot` read and write a slot's corners and light as
the editor does. The SDK's four `detail_object.part` files read and write back byte for byte.

`EditableAiMapTool` (`ESceneAIMapTool`) holds the AI map: the generator's settings (`EditableAiMapParams`, `SAIParams`),
the box node heights are packed across, and each `EditableAiMapNode` (`SAINode`: four three-byte links, the packed
plane normal, the cell and height, the editor's flags), with the editor's own settings and snap objects;
`get_node_position` unpacks a node as `UnpackPosition` does. The SDK's four `ai_map.part` files, GSC's `labx8` of 6,781
nodes among them, read and write back byte for byte.

## Typed objects

An object tool's part reads as an `EditableObjectPart<T>`: the GUID, `[main]`'s flags for the tools that keep them,
`[modif]`, and the objects `[object_N]` numbers. Each object is an `EditableToolObject` that reads its sections and
writes them as the editor's `SaveLTX` does, and names its tool's class, part name (`ClassName`), `[main]` version and
the key it is under (`version`, or the glow tool's `glow_tool_version`), and whether it keeps flags. A part refuses what the tool's `LoadLTX` refuses: another version, missing flags where the tool
keeps them, an object of another class. What a tool saves beyond that is kept as stored in the part's `tool_data`: the
light tool's `sun_shadow_dir`, `lcontrol_last_idx` and `[lcontrols]`, the scene object tool's `[appendrandom]`;
`EditableLightObject::read_controls` and `read_sun_direction` read the light tool's.

Groups (`group.part`, `group.part1` and on) hold each group as an `[object_N]` of class 0 and its members as
`[object_N_ingroup_M]`, with their own sections after them (`[object_N_ingroup_M_spawndata]`).
`EditableGroupTool::read_members` reads every member of one class as that class's type. A member keeps its own transform
in the level, and the editor loads it into its class's list before the objects of that tool's own part.

| Type                             | Editor              | Holds                                                                                  |
| -------------------------------- | ------------------- | -------------------------------------------------------------------------------------- |
| `EditableCustomObject`           | `CCustomObject`     | Every object's flags, name and transform                                               |
| `EditableObjectTransform`        | `CCustomObject`     | Position, rotation angles and scale, and the matrix they compose                       |
| `EditableShape`                  | `CEditShape`        | A shape's usage and its spheres and boxes, in the object's own space                   |
| `EditableParticleObject`         | `EParticlesObject`  | A placed effect: its `particles.xr` name and game modes                                |
| `EditableSoundSourceObject`      | `ESoundSource`      | A placed sound: file, own position, volume, distances, play windows                    |
| `EditableSoundEnvironmentObject` | `ESoundEnvironment` | A reverb volume: a shape whose first box separates an inner environment from outer     |
| `EditableFogVolumeObject`        | `EFogVolume`        | A fog body: a shape placed by its transform, an emitter or occlusion, a group          |
| `EditableGlowObject`             | `CGlow`             | A light flare: radius, shader, texture and the fixed-size flag                         |
| `EditableLightObject`            | `CLight`            | A light: colour and brightness, range, attenuation, flags, control, fuzzy points       |
| `EditableSceneObject`            | `CSceneObject`      | A placed library object: its `$objects$` reference and flags                           |
| `EditableSectorObject`           | `CSector`           | A sector: colour, default flag, the object meshes it holds, map index                  |
| `EditablePortalObject`           | `CPortal`           | A portal: the sectors in front and behind, and its polygon                             |
| `EditableShapeObject`            | `CEditShape`        | A shape tool's object, and what a spawn point attaches                                 |
| `EditableWayObject`              | `CWayObject`        | A patrol path: points with flags and names, each one's links                           |
| `EditableSpawnPointObject`       | `CSpawnPoint`       | A respawn point, an environment modifier, or an entity: its section, flags, spawn data |

Editor quirks kept:

- A sound source stands at its own `snd_position`. `ESoundSource` overrides `SetPosition`, so its object transform
  never moves.
- `EditableLevelValue::read_u32` reads a `w_u32` field printed with `%d`, as `r_u32` does, so the uninitialised high
  bits in GSC's sound source `flags` survive a round trip.
- A sound environment writes its own `version` (18) over the shape's (2); `CEditShape::LoadLTX` reads the shape's fields
  under whichever it finds.
- A way's links are keyed `link_wp_%4d_%4d`; the padding sorts them in their numbers' order, which is the order the
  editor appends them in.
- An entity's `[object_N_spawndata]` keeps its section (`name`) and flags (`fl`) apart, and its packet's numbered fields
  as stored; `xrf-rawdata` reads them as the entity's class.
- `EditableShapeObject::get_scaled_elements` is `ApplyScale`, which the build gives an entity's attached shape. A unit
  scale leaves the elements as stored: GSC's build kept a box's negative zero where `mul_43`'s sums would not.

### Transforms

`EditableObjectTransform::to_matrix` composes exactly as `CCustomObject::OnUpdateTransform` does: scale, then rotate by
`setXYZi` of the negated angles, then translate, through `Fmatrix::mul` in the editor's order of sums. It produces the
same bits the editor exported.

Going back from a matrix:

- `from_matrix` decomposes it.
- `round_as_saved` rounds a transform as its `%f` text reads back, which is what the editor composed from.
- A decomposed transform can land a whole `%f` step off the saved text. `find_saved` tries each angle and scale a step
  or two either way until the caller's test passes.
- `from_matrix_as_saved` finds the saved transform that composes into the matrix's exact bits, or rounds when none does.

The editor's own `%f` prints some floats one unit off in the last digit (`-25.461156` where the float is nearer
`-25.461157`). A typed write differs from the file there but reads back the same.

## Make a scene

A scene built in code takes the same types:

- `EditableLevelFile::new` writes every key `EScene::SaveLTX` does: the version, the GUID, level options
  (`EditableLevelOptions::new` gives a new level's), build parameters (`EditableBuildParams::DEFAULT` are the
  editor's), the build parameters' reserved fields as they hold them (zero in the editor's defaults, GSC's leftover
  memory in its scenes, which the Build copies into `build.prj`), the camera, and the owner tag.
- `EditableObjectPart::to_scene_tool` gives the tool a scene holds, named after the object's `TOOL_NAME`, with the part
  as its one text file; `EditableWallmarkTool::to_scene_tool` does the same with its binary part. Each type's
  `read_from_scene_tool` reads a scene's tool back, refusing another tool's or one of other than its one part.

A scene's sections are kept in the order they were set, and a read one's in its file's; both save to the same bytes, so
compare scenes by what they write.

## Errors

Reads refuse what the editor refuses or skips:

- a `.level` of a version other than 5;
- a scene or text part with no GUID;
- a binary part with no GUID chunk, a GUID chunk not 16 bytes long, or no tool data chunk;
- a part whose GUID is not its scene's.

`EditableTextPart::get_objects` refuses an object `[main]` counts but no section holds.

Writes refuse:

- what [xrf-ini](../xrf-ini/README.md) refuses, and text that would not read back as itself;
- a scene whose part directory holds a part file it would not write (a removed tool, or a numbered file past its last),
  since the editor would read it as part of the scene. Nothing is written in that case; `check_writable` runs that
  check alone, for a caller writing something else first.

## Checks

Run `cargo test --locked -p xrf-editable-level` from the repository root.

Sources: the [scene](src/scene/editable_level_scene.rs), the [parts](src/part/) and the [tests](src/tests/).
