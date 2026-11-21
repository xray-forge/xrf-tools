# xrf-editable-level

Reads and writes the Level Editor's scenes, the source a compiled level is built from.

A scene is a `.level` main file under `$maps$` plus a directory beside it, named after it, with one part file per
editor tool. The editor calls it `EScene`; this crate says "editable level", after
[xrf-editable-object](../xrf-editable-object/README.md), which reads the `.object` files a scene places.

A scene read as an `EditableLevelScene` writes its files back byte for byte. A typed tool's `to_part` writes its part's
two chunks in the editor's order, so a part stored in another order comes back reordered.

## Read a scene

```rust,no_run
use xrf_editable_level::{EditableLevelPart, EditableLevelScene};

fn main() -> xrf_error::XrfResult {
  let scene = EditableLevelScene::read_from_path(&"rawdata/levels/labx8.level")?;

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

`EditableLevelScene::read_from_source` finds a scene's parts as the editor does, over any `EditableScenePartSource`
that lists and reads the part directory: `read_from_path` gives it the host directory, and the app its mounted search.

- A part file starting with `[` is text. Any other is chunks, whose data chunk is the class of the tool the file is
  named after (`EditableBinaryPart::get_tool_class_id`), where it has one.
- A tool's numbered files (`group.part1`, `group.part2`) follow its first until one is missing.
- A file that does not read stops its tool there, so each part keeps its file's place. `read_from_source` records which
  file and why in `EditableSceneRead::files`; `read_from_path` refuses the scene.
- A part carrying another scene's GUID refuses the scene: the editor skips it, so the scene would open without that
  tool's objects.
- Files with other names, such as the editor's `.~part` backups, are not part of the scene.

### Text files

The `.level` and text parts are written as `CInifile::save_as` writes them, through [xrf-ini](../xrf-ini/README.md)'s
`IniFormat::XRAY_SAVE_AS`.

- Each keeps the whole `IniFile` as its `ini` field. Typed getters and setters work on top of it:
  `EditableLevelOptions`, `EditableBuildParams`, `EditableLevelCamera`, `EditablePartHeader`, and `EditablePartObject`
  for each object's class, name and transform.
- A spawn's data is its `[object_N_spawndata]` section: one numbered field per value its packet holds, multi-line
  custom data included.
- `EditableLevelValue` formats values as the editor does: `%f` floats and `x, y, z` vectors.

Typed values are for use; the kept text is what writes back. Not every saved number is an `f32` printed with `%f`:
`labx8.level`'s camera stands at `-80.052984`, which no `f32` prints as.

### Binary parts

The AI map, detail objects and wall marks are saved as chunks. An `EditableBinaryPart` keeps the tool's stream as
bytes, with its chunk order and any chunk the editor never writes in a `ChunkLayout`; a typed tool's `to_part` does not
carry that layout over.

Each typed tool keeps its own stream's chunks the same way:

- Every chunk it reads must be read whole.
- A chunk the tool does not save (xray-re's `0x7777` in all three) is kept in the tool's `layout` and written back.
- Detail objects and wall mark slots are numbered from 0 as the editor opens them, and the detail slots must fill the
  header's grid, on read and on write.

| Type                       | Editor               | Holds                                                            |
| -------------------------- | -------------------- | ---------------------------------------------------------------- |
| `EditableWallmarkTool`     | `ESceneWallmarkTool` | The wall mark brush and its decals in slots                      |
| `EditableDetailObjectTool` | `EDetailManager`     | Detail objects, the base texture's colour indices, the slot grid |
| `EditableAiMapTool`        | `ESceneAIMapTool`    | The generator's settings, the node height box, the nodes         |

- **Wall marks** read as `LoadStream` reads versions 3 and 4 and write as `SaveStream` writes version 4. Each
  `EditableWallmark` keeps its fan's lit vertices, box and sphere; `EditableWallmark::get_bounds` gives the ones
  `AddWallmark` does. An empty slot stays as stored; the editor drops it on load.
- **Detail objects:** each `EditableDetailObject` (`EDetail`) plants a library object by reference, with its scale
  range, density factor and flags. The grid keeps its header, box, base texture, density and snap objects, and its
  slots as stored (`DetailSlot`, four words a slot); `xrf-level`'s `LevelDetailsSlot` reads a slot's corners and
  light. The SDK's four `detail_object.part` files write back byte for byte.
- **AI map:** `EditableAiMapParams` is `SAIParams`. Each `EditableAiMapNode` (`SAINode`) holds four three-byte links,
  the packed plane normal, the cell and height as an `xrf-math` `AiNodePosition`, and the editor's flags; the editor's
  own settings and snap objects are kept too. `get_node_position` unpacks a node as `UnpackPosition` does. The SDK's
  four `ai_map.part` files, GSC's 6,781-node `labx8` among them, write back byte for byte.

## Typed objects

An object tool's part reads as an `EditableObjectPart<T>`: the GUID, `[main]`'s flags for the tools that keep them,
`[modif]`, and the objects `[object_N]` numbers. Each `T` is an `EditableToolObject`: it reads and writes its sections
as the editor's `SaveLTX` does, and names its tool's class (`CLASS_ID`), part name (`TOOL_NAME`, the editor's
`ClassName`), `[main]` version and the key it is under (`version`, or the glow tool's `glow_tool_version`), and whether
it keeps flags.

A part refuses what the tool's `LoadLTX` refuses: another version, missing flags where the tool keeps them, an object of
another class. What a tool saves beyond its objects stays as stored in the part's `tool_data`: the light tool's
`sun_shadow_dir`, `lcontrol_last_idx` and `[lcontrols]`, which `EditableLightObject::read_controls` and
`read_sun_direction` read, and the scene object tool's `[appendrandom]`.

Groups (`group.part`, `group.part1` and on) hold each group as an `[object_N]` of class 0 and its members as
`[object_N_ingroup_M]`, followed by their own sections (`[object_N_ingroup_M_spawndata]`).

- `EditableGroupTool::read_groups` reads each group as an `EditableCustomObject`.
- `read_grouped_members` reads every member of one class as that class's type, each an `EditableGroupMember` naming its
  group. A member keeps its own transform in the level; the editor loads it into its class's list before that tool's
  own objects.
- Members are `[object_N_ingroup_i]` for `i` in `0..ingroup_count`, taken by number as `EScene::ReadObjectsLTX` loads
  them, not in string order (`ingroup_10` sorts before `ingroup_2`). A member past the count, or a counted one missing,
  is refused.

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
- A sound environment writes its own `version` (18) over the shape's (2); `CEditShape::LoadLTX` reads the shape's
  fields under whichever it finds.
- A way's links are keyed `link_wp_%4d_%4d`; the padding sorts them in their numbers' order, the order the editor
  appends them in. A number past four digits, which the editor's `sscanf` asserts on, is refused.
  `EditableWayObject::check_links` refuses a link to its own point (which `CWayPoint::CreateLink` drops), past the
  way's points, or from or to a point past 9999, whose key four digits cannot spell.
- An entity's `[object_N_spawndata]` keeps its section (`name`) and flags (`fl`) apart, and its packet's numbered fields
  as stored; `xrf-rawdata` reads them as the entity's class.
- `EditableShapeObject::get_scaled_elements` is `ApplyScale`, which the build gives an entity's attached shape. A unit
  scale leaves the elements as stored: GSC's build kept a box's negative zero where `mul_43`'s sums would not.

### Transforms

`EditableObjectTransform::to_matrix` composes exactly as `CCustomObject::OnUpdateTransform` does, to the bits the
editor exported: scale, then rotate by `setXYZi` of the negated angles, then translate, through `Fmatrix::mul` in the
editor's order of sums.

Going back from a matrix:

- `from_matrix` decomposes it.
- `round_as_saved` rounds a transform as its `%f` text reads back, which is what the editor composed from.
- `find_saved` tries each angle and scale a `%f` step or two either way until the caller's test passes, since a
  decomposed transform can land a whole step off the saved text.
- `from_matrix_as_saved` finds the saved transform that composes into the matrix's exact bits, or rounds when none does.

The editor's own `%f` prints some floats one unit off in the last digit (`-25.461156` where the float is nearer
`-25.461157`). A typed write differs from the file there but reads back the same.

## Make a scene

A scene built in code takes the same types:

- `EditableLevelFile::new` writes every key `EScene::SaveLTX` does: version, GUID, level options
  (`EditableLevelOptions::new` gives a new level's), build parameters (`EditableBuildParams::DEFAULT` are the
  editor's), camera and owner tag. The build parameters' reserved fields are kept as they hold them: zero in the
  editor's defaults, GSC's leftover memory in its scenes, which the Build copies into `build.prj`.
- `EditableObjectPart::to_scene_tool` gives the tool a scene holds, named after the object's `TOOL_NAME`, with the part
  as its one text file. Each binary tool's `to_scene_tool` does the same with its binary part. Each type's
  `read_from_scene_tool` reads a scene's tool back, refusing another tool's or one without exactly one part.

A built scene keeps its sections in the order they were set, a read one in its file's order; both save to the same
bytes, so compare scenes by what they write.

## Errors

Reads refuse what the editor refuses or skips:

- a `.level` of a version other than 5;
- a scene or text part with no GUID;
- a binary part with no GUID chunk, a GUID chunk not 16 bytes long, or no tool data chunk;
- a part whose GUID is not its scene's, which refuses the whole scene;
- a group member past `ingroup_count` or missing under it, a way's link to its own point, a binary tool's chunk not
  read whole, numbered children with a gap, or detail slots that do not fill the grid.

`EditableTextPart::get_objects` refuses an object `[main]` counts but no section holds.

Writes refuse:

- what [xrf-ini](../xrf-ini/README.md) refuses, and text that would not read back as itself;
- a part file in the scene's directory that the scene would not write (a removed tool, or a numbered file past a
  tool's last), since the editor would read it as part of the scene; names compare without case. Nothing is written
  then, and `check_writable` runs this check alone, for a caller writing something else first;
- a scene tool whose name is not one file name, or two whose names differ only in case;
- detail slots that do not fill the grid.

## Checks

Run `cargo test --locked -p xrf-editable-level` from the repository root.

Sources: the [scene](src/scene/editable_level_scene.rs), the [parts](src/part/) and the [tests](src/tests/).
