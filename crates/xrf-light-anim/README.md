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
  let item = &restored.items[0];

  assert_eq!(restored, library);
  assert_eq!(item.get_duration_seconds(), Some(2.0));
  assert_eq!((item.keys[0].get_red(), item.keys[0].get_green(), item.keys[0].get_blue()), (0xFF, 0x80, 0x00));

  Ok(())
}
```

Read a library with `read_from_path`, `read_from_file`, `read_from_bytes` for an archived entry, or `read_from_chunk`.
`write` emits the library as top-level chunks in the writer's buffer.

## Library, items and keys

The library is a chunk tree: a `u16` version (chunk 0) and an items chunk (1) holding one child per animation, numbered
from zero. A file with no version chunk reads as version 0, as `ELightAnimLibrary::Load` treats one.

`LightAnimItem` is `CLAItem`: a name, its own frames per second, a frame count, and its keys in frame order, from a
common chunk (1) and a keys chunk (2). `get_duration_seconds` is the frame count over the rate, or `None` for a rate
that is not positive. `LightAnimKey` is a frame and a stored `u32` colour; a key holds its colour from its frame, and
the engine divides the frame by the item's rate to reach a time.

The colour's channels are read as the engine leaves them after load: alpha, red, green and blue from the high byte
down, through `get_alpha`, `get_red`, `get_green` and `get_blue`. A version 0 library stores red and blue swapped,
which the engine corrects on load and then saves as version 1. `is_bgr` reports such a library; the crate keeps the
stored value and the stored version, so swap the channels yourself before reading one that way.

`get_keys_count` counts keys across every animation.

## Errors and checks

Reads reject a file that cannot be opened, a missing items chunk, an animation missing its common or keys chunk, a key
count the keys chunk cannot hold, and bytes left in a keys chunk. A library read and written back is byte for byte the
file it came from when it carries a version chunk; the writer always writes one.

Run `cargo test --locked -p xrf-light-anim` from the repository root. See the [library](src/light_anim_file.rs),
[animation](src/light_anim_item.rs), [key](src/light_anim_key.rs), and [format tests](src/tests.rs).
