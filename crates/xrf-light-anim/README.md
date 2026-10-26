# xrf-light-anim

Reads and writes `.xr` light animation libraries, `lanims.xr` in the game: named colour animations, each a run of
colour keys over frames, which lamps and other lights name to pulse or flicker. `LightAnimFile` holds the library;
[xrf-visual](../xrf-visual/README.md)'s light packer resolves level lights' animations against it, and the desktop app's
level viewer and archive describer read it.

## Read and write a library

```rust
use xrf_chunk::{ChunkWriter, XRayByteOrder};
use xrf_light_anim::{LightAnimFile, LightAnimItem, LightAnimKey};

fn main() -> xrf_error::XrfResult {
  let library = LightAnimFile {
    version: 1,
    items: vec![LightAnimItem {
      name: "koster_01".into(),
      fps: 15.0,
      frame_count: 30,
      keys: vec![
        LightAnimKey { frame: 0, color: 0xFF_FF8000 },
        LightAnimKey { frame: 15, color: 0xFF_802000 },
      ],
    }],
  };

  let mut writer = ChunkWriter::new();
  library.write::<XRayByteOrder>(&mut writer)?;

  let restored = LightAnimFile::read_from_bytes::<XRayByteOrder>(writer.buffer)?;
  let item = restored.find_item("koster_01").expect("the animation is in the library");
  let keys = item.to_loaded_keys(restored.is_bgr());

  assert_eq!(restored, library);
  assert_eq!(item.get_duration_seconds(), Some(2.0));
  assert_eq!((keys[0].get_red(), keys[0].get_green(), keys[0].get_blue()), (0xFF, 0x80, 0x00));

  Ok(())
}
```

Read a library with `read_from_path`, `read_from_file`, `read_from_bytes` for an archived entry, or `read_from_chunk`.
`write` emits the library as top-level chunks in the writer's buffer.

## Library, items and keys

The library is a chunk tree: a `u16` version (chunk 0) and an items chunk (1) holding one child per animation, numbered
from zero. A file with no version chunk reads as version 0, as `ELightAnimLibrary::Load` treats one. Animations are
read as the engine opens them, chunk 0, 1, 2 and on until the first number the list lacks, so one numbered past a gap
is not read. `find_item(name)` is `FindItem`: the first animation of exactly that name, case included, and none for an
empty name.

`LightAnimItem` is `CLAItem`: a name, its own frames per second, a frame count, and its keys as the file lists them,
from a common chunk (1) and a keys chunk (2). `get_duration_seconds` is the frame count over the rate, or `None` for a
rate that is not positive. `LightAnimKey` is a signed frame and a stored `u32` colour. Frames are the engine's `int`,
and every shipped library keys some before zero: `svet_01` at `-1`, GAMMA's `koster_00_old` at `-2`.

The engine holds an animation's keys in a map, so `to_loaded_keys(is_bgr)` returns them as it does after load: by
frame, a frame listed twice keeping the colour read last. `normalize_keys(is_bgr)` stores them that way.

The colour's channels are read as the engine leaves them after load: alpha, red, green and blue from the high byte
down, through `get_alpha`, `get_red`, `get_green` and `get_blue`. A version 0 library stores red and blue swapped,
which the engine corrects on load and then saves as version 1. `is_bgr` reports such a library; the crate keeps the
stored value and the stored version, and `to_loaded_keys(file.is_bgr())` (or `LightAnimKey::to_swapped_red_blue`)
gives the colours the engine uses. `LightAnimFile::normalize` turns the whole library into what `Load` holds and `Save`
writes: version 1, colours as RGB, every animation's keys by frame.

`get_keys_count` counts keys across every animation.

## Evaluate an animation

```rust
use xrf_light_anim::LightAnimItem;

let mut item = LightAnimItem::new("pulse");

item.resize(10).expect("ten frames");
item.insert_key(10, 0xFF_FFFFFF).expect("the last frame is keyed");

// Halfway from transparent black to opaque white, every channel floored as the engine does.
assert_eq!(item.interpolate_rgb(5), Some(0x7F_7F7F7F));
assert_eq!(item.calculate_rgb(0.5), Some((7, item.interpolate_rgb(7).unwrap())));
assert_eq!(item.get_length_ms(), Some(666));
```

These are `CLAItem`'s own functions, in its `f32` arithmetic, over the keys as loaded:

- `interpolate_rgb(frame)` is `InterpolateRGB`: a key's colour on its frame, the last key's past the last, and between
  two keys both blended, alpha included, each channel taken to `byte / 255`, mixed linearly and floored back to a byte.
  `interpolate_bgr` exchanges red and blue in the result, as lamps take it.
- `calculate_rgb(seconds)` and `calculate_bgr` are `CalculateRGB` and `CalculateBGR`: the frame
  `floor((seconds mod length) * fps)`, looping and negative for a negative time, and its colour.
- `get_length_ms` is `Length_ms`, what the script class `color_animator` answers from `length()`.

Where the engine asserts or reads past its keys, these answer `None`: a frame past the frame count, an animation with no
keys, a frame before every key, and a rate that is not positive. They read colours as stored, so a version 0 library is
normalized first. The renderer draws lights with its own `f32` copy of this, which its tests hold to these within the
one step the engine's final floor takes.

## Edit a library

The editing operations are the SDK editor's (`TfrmEditLightAnim` over `CLAItem` and `ELightAnimLibrary`), and each
stores the keys as loaded, by frame and one a frame:

- `LightAnimItem::new(name)` starts an animation as `AppendItem` does: 15 frames a second, one frame, and a transparent
  black key at frame 0.
- `insert_key(frame, color)` keys a frame, replacing a key there; `delete_key(frame)` removes one, never the key at
  frame 0; `move_key(from, to)` moves one over whatever `to` held. Each refuses a frame past the frame count, which the
  engine asserts on. A move onto its own frame changes nothing, where the engine would erase the key.
- `resize(frame_count)` grows by moving the key at the old last frame to the new one, and shrinks by dropping every key
  from the new count on; it refuses zero frames.
- `get_previous_key_frame` and `get_next_key_frame` step between keys as the editor's buttons do.
- `LightAnimFile::append_item(name)`, `clone_item(source, name)`, `rename_item(from, to)` and `remove_item(name)` keep
  names unique, since `FindItem` only ever reaches the first animation of a name; `find_item_mut` edits one in place.

Write the result with `write`, which keeps whatever version the library carries; `normalize` first to save as the
engine does.

## Errors and checks

Reads reject a file that cannot be opened, a missing items chunk, an animation missing its common or keys chunk, a key
count the keys chunk cannot hold, and bytes left in a keys chunk. A library read and written back is byte for byte the
file it came from when it carries a version chunk and numbers its animations from zero without a gap; the writer
always writes a version and renumbers from zero. Every `lanims.xr` in the reference trees (vanilla, CoC, CS, Enhanced
Edition, Anomaly, GAMMA, Gunslinger) is version 1, numbered without a gap, and holds no frame twice.

Run `cargo test --locked -p xrf-light-anim` from the repository root. See the [library](src/light_anim_file.rs),
[animation](src/light_anim_item.rs), [key](src/light_anim_key.rs), and the tests of the [format](src/tests/format.rs),
[evaluation](src/tests/evaluation.rs) and [editing](src/tests/editing.rs).
