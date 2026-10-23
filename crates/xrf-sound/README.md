# xrf-sound

Sound files the engine plays, and the environment presets it mixes them in. `SoundFile` reads an Ogg/Vorbis sound's
headers and the X-Ray source parameters stored in its first Vorbis comment, with optional validation of the full audio
stream. `SoundEnvironmentFile` reads and writes `senvironment.xr`, the reverb preset library. The sound verifier and the
desktop app's archive previews read through it.

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

| Methods                                               | Work performed                                          |
| ----------------------------------------------------- | ------------------------------------------------------- |
| `read_from_path`, `read_from_bytes`                   | Read Vorbis headers and X-Ray metadata                  |
| `read_strictly_from_path`, `read_strictly_from_bytes` | Also decode the audio stream to detect payload failures |

A header-only read proves a structurally valid Ogg/Vorbis header set (identification, comment and setup packets of one
stream, Vorbis version 0, nonzero channels and sample rate), not a valid audio payload. Strict reads validate the audio
but still return metadata, not decoded samples.

## X-Ray parameters

`SoundMetadata::XRay` carries the comment version (`V1`, `V2`, or `V3`) and the resulting `XRaySoundParameters`: minimum
distance, maximum distance, base volume, game type, and maximum AI distance. The first Vorbis comment is read as a
little-endian `u32` version followed by that version's fields. Version 1 stores no base volume, which reads as 1.0;
versions 1 and 2 store no maximum AI distance, which reads as the maximum distance.

`SoundMetadata::EngineDefaults` means the sound has no comment or its first comment's version is not one of the three,
so the engine applies its built-in source parameters. Keep it distinct from an authored parameter block: it describes
the absence of recognized metadata, not an explicit set of values written by the sound author.

## Sound environments

`SoundEnvironmentFile` reads `senvironment.xr` with `read_from_path`, `read_from_file`, `read_from_bytes` or
`read_from_chunk`, taking the byte order as a type parameter like the other format crates. The library has no header:
it is a flat run of chunks numbered from zero, one `SoundEnvironment` preset each, and `write` emits them that way, so
a library read and written back is byte for byte the file it came from. A preset is a `u32` version, a name, and twelve
reverb parameters; version 4 and above add the EAX preset number in `environment`. `find_environment` matches a name
ignoring ASCII case, as the engine does.

## Errors and checks

Public sound reads wrap file, header, metadata, and decoding failures in `XrfError::Verify` with source context. A
recognized comment that is truncated, holds a non-finite parameter, or a maximum or maximum AI distance below 0.1 fails
the read. Environment reads fail on a preset older than version 3, which the engine skips, and on a preset that does
not fill its chunk. The crate does not play, encode, or edit sounds.

Run `cargo test --locked -p xrf-sound` from the repository root.

See the [read API](src/sound_file.rs), the [X-Ray comment format](src/sound_file_metadata.rs), the
[environment library](src/sound_environment_file.rs), and its [presets](src/sound_environment.rs).
