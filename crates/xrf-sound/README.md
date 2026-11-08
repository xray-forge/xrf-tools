# xrf-sound

The sound files the engine plays, and the environment presets it mixes them in.

| Type                   | Does                                                                                    |
| ---------------------- | --------------------------------------------------------------------------------------- |
| `SoundFile`            | Reads an Ogg/Vorbis sound's headers, length and X-Ray parameters; writes new parameters |
| `VorbisDecoder`        | Decodes a sound's audio, seeking to the frame                                           |
| `SoundEnvironmentFile` | Reads, edits and writes `senvironment.xr`, the reverb preset library                    |

The sound verifier, `xrf-audio` and the desktop app's archive previews read through it. It does not play or encode
sounds.

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

The path must be an Ogg/Vorbis sound. For an archive entry, read its bytes through the VFS and use the byte-based
method; nothing needs extracting first.

### Validation depth

| Methods                                               | Work performed                                          |
| ----------------------------------------------------- | ------------------------------------------------------- |
| `read_from_path`, `read_from_bytes`                   | Read Vorbis headers, length and X-Ray metadata          |
| `read_strictly_from_path`, `read_strictly_from_bytes` | Also decode the audio stream to detect payload failures |

A header-only read proves that libvorbis would open the headers:

- identification, comment and setup packets of one stream;
- Vorbis version 0, nonzero channels and sample rate;
- block sizes of 64 to 8192 samples, with framing bits set;
- every comment within its packet.

It does not prove the audio payload is valid. Strict reads check the audio, but still return metadata, not samples.

### Length

`samples_count` is `ov_pcm_total`: the granule position of the stream's last page, found from the end of the file
without reading the audio in between. `get_duration` divides it by the sample rate, as the engine's `length_sec` does.

## X-Ray parameters

`SoundMetadata::XRay` carries the comment version (`V1`, `V2` or `V3`) and the resulting `XRaySoundParameters`: minimum
distance, maximum distance, base volume, game type, and maximum AI distance.

- The first Vorbis comment is read as a little-endian `u32` version, then that version's fields.
- Version 1 stores no base volume; it reads as 1.0.
- Versions 1 and 2 store no maximum AI distance; it reads as the maximum distance.

`SoundMetadata::EngineDefaults` means the engine applies its built-in parameters, because the sound has:

- no comment;
- a first comment whose version is not one of the three;
- or a first comment cut short (GAMMA's re-encoded sounds carry two bytes, which the engine reads past).

It describes the absence of recognized metadata, not values the author wrote. `get_parameters` returns the parameters
the sound plays with either way. `XRaySoundParameters::default()` is the engine's set, which the SDK's sound editor
also starts a sound with.

`XRaySoundGameType` is what the sound means to the AI, an `ESoundTypes` mask. Its named values are the SDK editor's
list and labels; a mask the SDK has no name for reads as `Unknown` and is written back unchanged.

## Edit sound parameters

`SoundFile::write_parameters` returns a sound's bytes with new parameters, in the version 3 block the SDK's
`MakeGameSound` writes.

- A first comment that is an X-Ray block is replaced. Any other first comment, and every later one, follows the new
  block.
- Audio packets and their granule positions are kept; the pages are laid out again.
- `XRaySoundParameters::validate` refuses what the engine refuses to load.
- `clamp` holds each value to the ranges the SDK's sound editor offers, with the maximum AI distance within the maximum
  distance.

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

The SDK's sound thumbnails, which keep the parameters beside a source `.wav` before encoding, are not read here.

## Decode a sound

`VorbisDecoder` decodes an in-memory Ogg/Vorbis sound packet by packet, for `xrf-audio` to play.

- It reads the Ogg pages itself and hands each audio packet to symphonia's Vorbis codec; comments are never
  interpreted.
- Each packet's start frame is placed as libvorbis places it: counted from the stream's start (the first packet renders
  nothing), and after a seek, from the granule position of the page it lands on.
- Decoding stops at `get_samples_count` (the same `ov_pcm_total` as `SoundFile::samples_count`), trimming the last
  packet to it.

```rust,no_run
use xrf_sound::VorbisDecoder;

fn main() -> xrf_error::XrfResult {
  let bytes: Vec<u8> = std::fs::read("gamedata/sounds/music/outro.ogg")?;
  let mut decoder = VorbisDecoder::open(bytes.as_slice())?;
  let mut samples: Vec<f32> = Vec::new();

  decoder.seek(48_000)?;

  while let Some(first_frame) = decoder.decode_next(&mut samples)? {
    // A packet may start before the frame sought; drop what comes before it by `first_frame`.
    println!("{} frames from {first_frame}", samples.len() / usize::from(decoder.get_channels()));
  }

  Ok(())
}
```

- `VorbisDecoder::is_ogg` recognizes an Ogg stream by its first bytes, for a player choosing a decoder.
- `seek` bisects the bytes by granule position and lands on the last page that places a frame at or before the target
  and from which a reader receives a packet. Packets up to that page's end are decoded and dropped.
- A packet the codec refuses as corrupt is skipped; the packets after it wait for the next page's granule position.

The `fixtures` feature exports `VorbisNoiseFixture`: an Ogg/Vorbis noise sound written without an encoder (one codebook
pair, a flat floor, random residues; short and long blocks, a packet over three pages, a trimmed last page). This
crate's decoder tests and `xrf-audio`'s seek tests compare against it frame for frame.

## Sound environments

`SoundEnvironmentFile` reads `senvironment.xr` with `read_from_path`, `read_from_file`, `read_from_bytes` or
`read_from_chunk`, taking the byte order as a type parameter.

- The library has no header: it is a flat run of chunks numbered from zero, one `SoundEnvironment` preset each.
- `write` writes them the same way, so a library read and written back is byte for byte the original.

### Presets

A preset is a `u32` version, a name, and twelve reverb parameters. Version 4 adds the EAX room it was authored from, in
`environment` (an `EaxEnvironment`).

- The version follows from that field alone (`get_version`).
- `normalize` turns a version 3 preset into the version 4 the engine saves, with the generic room.
- The EAX room only records authoring: the SDK filled the parameters from it through the sound card's driver, and the
  engine plays the parameters alone.

Engine operations on a preset:

- `SoundEnvironment::new` and `set_default` give the EAX listener defaults.
- `set_identity` silences the room, as the engine does where a level marks no environment.
- `clamp` holds each parameter to the range EAX accepts (`get_parameter_ranges`).
- `lerp` blends toward another preset, as the engine moves the listener's environment each frame.

### The library

- A level finds a preset by name, ignoring ASCII case as the engine does (`find_environment`); its compiled sound
  geometry finds one by number (`get_environment_index`).
- The SDK's library operations are `append_environment`, `clone_environment`, `rename_environment` and
  `remove_environment`.
- Names must be non-empty and unique ignoring case, since the engine takes the first match.
- Removing a preset renumbers those after it.

## Errors

- Sound reads and parameter writes wrap file, header, metadata and decoding failures in `XrfError::Verify`, with source
  context.
- A recognized comment that is truncated, holds a non-finite parameter, or has a maximum or maximum AI distance below
  0.1 fails the read.
- Environment reads fail on a preset of a version other than 3 or 4, or one that does not fill its chunk. The engine
  skips a preset older than 3 and reads a later one as 4, which would not write back as it was.

## Checks

Run `cargo test --locked -p xrf-sound` from the repository root.

Sources: the [read and write API](src/sound_file.rs), the [decoder](src/vorbis/vorbis_decoder.rs), the
[X-Ray comment format](src/xray/xray_sound_parameters.rs), the
[environment library](src/environment/sound_environment_file.rs), and its
[presets](src/environment/sound_environment.rs).
