# xrf-sound

Reads Ogg/Vorbis sound headers and X-Ray source parameters, with optional validation of the full audio stream.

## Read sound metadata

```rust,no_run
use xrf_sound::{SoundFile, SoundMetadata};

fn main() -> xrf_error::XrfResult {
  let sound = SoundFile::read_from_path("gamedata/sounds/weapons/shot.ogg")?;

  println!("{} channels at {} Hz", sound.channels, sound.sample_rate);

  match sound.metadata {
    SoundMetadata::EngineDefaults => println!("Uses engine sound parameters"),
    SoundMetadata::XRay { version, parameters } => {
      println!("{version:?}: maximum distance {}", parameters.max_distance);
    }
  }

  Ok(())
}
```

The path must identify an Ogg/Vorbis sound. For an archive entry, read the bytes through the VFS and use the byte-based
counterpart; an archived asset does not need to be extracted first.

## Choose the validation depth

Call `SoundFile::read_from_path` or `read_from_bytes` for channels, sample rate, and `SoundMetadata`. Use
`read_strictly_from_path` or `read_strictly_from_bytes` to also decode and validate the payload.

| Methods                                               | Work performed                                          |
| ----------------------------------------------------- | ------------------------------------------------------- |
| `read_from_path`, `read_from_bytes`                   | Read Vorbis headers and X-Ray metadata                  |
| `read_strictly_from_path`, `read_strictly_from_bytes` | Also decode the audio stream to detect payload failures |

A header-only read does not prove the audio payload is valid. Sounds without a recognized X-Ray comment use the
engine's default source parameters. Strict reads validate audio but still return metadata, not decoded samples.

## X-Ray parameters

`SoundMetadata::XRay` carries the recognized comment version (`V1`, `V2`, or `V3`) and the resulting parameters:
minimum distance, maximum distance, base volume, game type, and maximum AI distance. The reader interprets the first
Vorbis comment according to its X-Ray version.

Keep `EngineDefaults` distinct from an authored parameter block: it describes the absence of recognized metadata, not
an explicit set of values written by the sound author.

## Errors and checks

Public sound reads wrap file, header, metadata, and decoding failures in `XrfError::Verify` with source context.
Unrecognized X-Ray metadata can use defaults; malformed headers or truncated recognized metadata can still fail.
The crate does not play, encode, or edit sounds.

Run `cargo test --locked -p xrf-sound` from the repository root.

See the [read API](src/sound_file.rs) and [X-Ray comment format](src/sound_file_metadata.rs).
