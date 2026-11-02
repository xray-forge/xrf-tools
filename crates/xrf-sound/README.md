# xrf-sound

Sound files the engine plays, and the environment presets it mixes them in. `SoundFile` reads what the engine reads of
an Ogg/Vorbis sound — its stream headers, its length, and the X-Ray source parameters stored in its first Vorbis comment
— with optional validation of the full audio stream, and writes new parameters into a sound. `SoundEnvironmentFile`
reads, edits and writes `senvironment.xr`, the reverb preset library. The sound verifier and the desktop app's archive
previews read through it.

## Read sound metadata

```rust,no_run
use xrf_sound::{SoundFile, SoundMetadata};

fn main() -> xrf_error::XrfResult {
  let sound = SoundFile::read_from_path("gamedata/sounds/weapons/shot.ogg")?;

  println!("{} channels at {} Hz for {:?}", sound.channels, sound.sample_rate, sound.get_duration());

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
| `read_from_path`, `read_from_bytes`                   | Read Vorbis headers, length and X-Ray metadata          |
| `read_strictly_from_path`, `read_strictly_from_bytes` | Also decode the audio stream to detect payload failures |

A header-only read proves the header set libvorbis opens: identification, comment and setup packets of one stream,
Vorbis version 0, nonzero channels and sample rate, block sizes of 64 to 8192 samples, set framing bits, and every
comment within its packet. It does not prove a valid audio payload. Strict reads validate the audio but still return
metadata, not decoded samples.

`samples_count` is `ov_pcm_total`, the granule position of the stream's last page, found from the end of the file
without reading the audio between; `get_duration` divides it by the sample rate, as the engine's `length_sec` does.

## X-Ray parameters

`SoundMetadata::XRay` carries the comment version (`V1`, `V2`, or `V3`) and the resulting `XRaySoundParameters`: minimum
distance, maximum distance, base volume, game type, and maximum AI distance. The first Vorbis comment is read as a
little-endian `u32` version followed by that version's fields. Version 1 stores no base volume, which reads as 1.0;
versions 1 and 2 store no maximum AI distance, which reads as the maximum distance.

`SoundMetadata::EngineDefaults` means the sound has no comment or its first comment's version is not one of the three,
so the engine applies its built-in source parameters. Keep it distinct from an authored parameter block: it describes
the absence of recognized metadata, not an explicit set of values written by the sound author. `get_parameters` answers
the parameters the sound plays with either way; `XRaySoundParameters::default()` is the engine's set, which the SDK's
sound editor also starts a sound with.

`XRaySoundGameType` is what the sound means to the AI, an `ESoundTypes` mask. Its named values are the SDK editor's
list with its labels; a mask the SDK has no name for reads as `Unknown` and is written back unchanged.

## Edit sound parameters

`SoundFile::write_parameters` returns a sound's bytes with new parameters in the version 3 block the SDK's
`MakeGameSound` encodes a sound with. A first comment that is an X-Ray block is replaced; any other first comment, and
every later one, stays behind the new block. The audio packets and their granule positions are kept as they were, and
the pages are laid out again. `XRaySoundParameters::validate` refuses what the engine refuses to load, and `clamp` holds
each value to the ranges the SDK's sound editor offers, the maximum AI distance within the maximum distance.

```rust,no_run
use xrf_sound::{SoundFile, XRaySoundGameType};

fn main() -> xrf_error::XrfResult {
  let bytes: Vec<u8> = std::fs::read("gamedata/sounds/weapons/shot.ogg")?;
  let mut parameters = SoundFile::read_from_bytes(&bytes)?.metadata.get_parameters();

  parameters.max_distance = 150.0;
  parameters.game_type = XRaySoundGameType::WeaponShooting;
  parameters.clamp();

  std::fs::write("shot.ogg", SoundFile::write_parameters(&bytes, &parameters)?)?;

  Ok(())
}
```

The crate does not play or encode sounds, and the SDK's sound thumbnails, which hold the parameters beside a source
`.wav` before encoding, are not read here.

## Sound environments

`SoundEnvironmentFile` reads `senvironment.xr` with `read_from_path`, `read_from_file`, `read_from_bytes` or
`read_from_chunk`, taking the byte order as a type parameter like the other format crates. The library has no header:
it is a flat run of chunks numbered from zero, one `SoundEnvironment` preset each, and `write` emits them that way, so
a library read and written back is byte for byte the file it came from. A preset is a `u32` version, a name, and twelve
reverb parameters; version 4 adds the EAX room it was authored from in `environment`, an `EaxEnvironment`. The version
follows from that field alone (`get_version`), and `normalize` turns a version 3 preset into the version 4 the engine
saves, with the generic room.

A level reaches a preset by name, matched ignoring ASCII case as the engine does (`find_environment`), and its compiled
sound geometry by number (`get_environment_index`). The SDK's library operations are `append_environment`,
`clone_environment`, `rename_environment` and `remove_environment`; names must be nonempty and unique ignoring case,
since the engine takes the first match. Removing a preset renumbers those after it.

A preset carries the engine's own operations: `SoundEnvironment::new` and `set_default` give the EAX listener defaults,
`set_identity` silences the room as the engine does where a level marks no environment, `clamp` holds each parameter to
the range EAX accepts (`get_parameter_ranges`), and `lerp` blends towards another preset as the engine moves the
listener's environment each frame. The EAX room only records authoring: the SDK filled the parameters from it through
the sound card's driver, and the engine plays the parameters alone.

## Errors and checks

Public sound reads and parameter writes wrap file, header, metadata, and decoding failures in `XrfError::Verify` with
source context. A recognized comment that is truncated, holds a non-finite parameter, or a maximum or maximum AI
distance below 0.1 fails the read. Environment reads fail on a preset of a version other than 3 or 4, and on a preset
that does not fill its chunk: the engine skips one older than 3 and reads a later one as 4, which would not write back
as it was.

Run `cargo test --locked -p xrf-sound` from the repository root.

See the [read and write API](src/sound_file.rs), the [X-Ray comment format](src/xray/xray_sound_parameters.rs), the
[environment library](src/environment/sound_environment_file.rs), and its [presets](src/environment/sound_environment.rs).
