# xrf-light-anim

Reads, writes, evaluates and edits light animation libraries (`lanims.xr` in the game): named colour animations that
lamps and other lights use to pulse or flicker.

`LightAnimFile` holds the library. [xrf-visual](../xrf-visual/README.md)'s light packer resolves level lights'
animations against it, and the desktop app's level viewer and archive describer read it.

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

- Read with `read_from_path`, `read_from_file`, `read_from_bytes` (for an archived entry), or `read_from_chunk`.
- `write` emits the library as top-level chunks in the writer's buffer.

## The data

| Type            | Engine              | Holds                                                  |
| --------------- | ------------------- | ------------------------------------------------------ |
| `LightAnimFile` | `ELightAnimLibrary` | A `u16` version (chunk 0) and the animations (chunk 1) |
| `LightAnimItem` | `CLAItem`           | Name, its own frames per second, frame count, and keys |
| `LightAnimKey`  | —                   | A signed frame and a stored `u32` colour               |

Library:

- A file with no version chunk reads as version 0, as `ELightAnimLibrary::Load` treats it.
- Animations are children numbered from zero. They are read as the engine reads them, until the first missing number,
  so one numbered past a gap is not read.
- `find_item(name)` is `FindItem`: the first animation with exactly that name, case included; none for an empty name.
- `get_keys_count` counts keys across every animation.

Animations and keys:

- An animation is a common chunk (1) and a keys chunk (2).
- `get_duration_seconds` is frame count over rate, or `None` for a rate that is not positive.
- Frames are the engine's `int`, and every shipped library keys some before zero: `svet_01` at `-1`, GAMMA's
  `koster_00_old` at `-2`.
- The engine keeps keys in a map. `to_loaded_keys(is_bgr)` returns them as it holds them after load: by frame, a frame
  listed twice keeping the colour read last. `normalize_keys(is_bgr)` stores them that way.

### Colours

Channels are read as the engine has them after load, high byte down: `get_alpha`, `get_red`, `get_green`, `get_blue`.

A version 0 library stores red and blue swapped; the engine fixes them on load and saves version 1.

- `is_bgr` reports such a library. The crate keeps the stored colours and version.
- `to_loaded_keys(file.is_bgr())`, or `LightAnimKey::to_swapped_red_blue`, gives the colours the engine uses.
- `LightAnimFile::normalize` converts the whole library to what `Load` holds and `Save` writes: version 1, RGB colours,
  keys by frame.

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

These port `CLAItem`'s own functions, in its `f32` arithmetic, over the keys as loaded:

| Method                                    | Engine                         | Returns                                                        |
| ----------------------------------------- | ------------------------------ | -------------------------------------------------------------- |
| `interpolate_rgb(frame)`                  | `InterpolateRGB`               | The colour at a frame                                          |
| `interpolate_bgr(frame)`                  | —                              | The same with red and blue exchanged, as lamps take it         |
| `calculate_rgb(seconds)`, `calculate_bgr` | `CalculateRGB`, `CalculateBGR` | The frame `floor((seconds mod length) * fps)` and its colour   |
| `get_length_ms`                           | `Length_ms`                    | What the script class `color_animator` answers from `length()` |

- On a key's frame the colour is the key's; past the last key it is the last key's.
- Between two keys, every channel (alpha included) becomes `byte / 255`, is mixed linearly, and is floored back to a
  byte.
- `calculate_*` loops, and gives a negative frame for a negative time.
- Where the engine asserts or reads past its keys, these answer `None`: a frame past the frame count, no keys, a frame
  before every key, or a rate that is not positive.
- They read colours as stored, so normalize a version 0 library first.

The renderer draws lights with its own `f32` copy of this; its tests keep it within one floor step of these results.

## Edit a library

The editing operations match the SDK editor (`TfrmEditLightAnim` over `CLAItem` and `ELightAnimLibrary`). Each keeps
keys as loaded: by frame, one per frame.

Animations:

- `LightAnimItem::new(name)` starts one as `AppendItem` does: 15 frames a second, one frame, and a transparent black key
  at frame 0.
- `resize(frame_count)` grows by moving the key at the old last frame to the new last frame, and shrinks by dropping
  every key from the new count on. It refuses zero frames.

Keys:

- `insert_key(frame, color)` keys a frame, replacing a key already there.
- `delete_key(frame)` removes one, never the key at frame 0.
- `move_key(from, to)` moves one over whatever `to` held. Moving onto its own frame changes nothing (the engine would
  erase the key).
- All three refuse a frame past the frame count, which the engine asserts on.
- `get_previous_key_frame` and `get_next_key_frame` step between keys like the editor's buttons.

Library:

- `LightAnimFile::append_item(name)`, `clone_item(source, name)`, `rename_item(from, to)` and `remove_item(name)` keep
  names unique, since `FindItem` only reaches the first animation with a name.
- `find_item_mut` edits one in place.

`write` keeps the library's version. Call `normalize` first to save as the engine does.

## Errors

Reads reject:

- a file that cannot be opened;
- a missing items chunk;
- an animation missing its common or keys chunk;
- a key count the keys chunk cannot hold, or bytes left over in it.

A library writes back byte for byte when it has a version chunk and numbers its animations from zero without a gap; the
writer always writes a version and renumbers from zero. Every `lanims.xr` in the reference trees (vanilla, CoC, CS,
Enhanced Edition, Anomaly, GAMMA, Gunslinger) is version 1, numbered without a gap, with no frame keyed twice.

## Checks

Run `cargo test --locked -p xrf-light-anim` from the repository root.

Sources: the [library](src/light_anim_file.rs), [animation](src/light_anim_item.rs), [key](src/light_anim_key.rs), and
the tests of the [format](src/tests/format.rs), [evaluation](src/tests/evaluation.rs) and
[editing](src/tests/editing.rs).
