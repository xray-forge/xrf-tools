# xrf-audio

Plays sound for the XRF tools: a mixer of preallocated voices on the audio thread, the control side that steers it,
decoding and streaming of the game's sounds, and the output device. It depends on no Tauri and no VFS: the caller hands
it bytes, so an editor, a CLI or a game can use it.

## Play a sound

`AudioEngineParts::new` makes the two halves. The `AudioEngine` stays with the caller; the `AudioMixer` goes to an
`AudioOutput`, which plays it on a device, or to an `OfflineRender`, which renders it without one.

```rust,no_run
use std::sync::Arc;

use xrf_audio::{
  AudioBus, AudioEngineParts, AudioEngineSettings, AudioOutput, AudioOutputChoice, AudioPlay, DecodedSound,
  SoundDecoder, SoundStream, SoundStreamStart, VoiceSource,
};

fn main() -> xrf_error::XrfResult {
  let AudioEngineParts { mut engine, mixer } = AudioEngineParts::new(AudioEngineSettings::default());
  let _output = AudioOutput::start(mixer, AudioOutputChoice::SystemDefault)?;

  // A short sound, decoded once and shared by every voice that plays it.
  let shot: Arc<[u8]> = std::fs::read("gamedata/sounds/weapons/shot.ogg")?.into();
  let sound = Arc::new(DecodedSound::decode(shot)?);
  let source = VoiceSource::Decoded { sound, start_frame: 0, is_looped: false };

  engine.play(AudioPlay::new(source, AudioBus::World)).expect("a voice is free");

  // A long one, decoded on a thread of its own as it plays.
  let music: Arc<[u8]> = std::fs::read("gamedata/sounds/music/credits.ogg")?.into();
  let format = SoundDecoder::open(music.clone())?.get_format();
  let reader = SoundStream::start(music, format, SoundStreamStart { frame: 0, is_looped: true })?;
  let voice = engine.play(AudioPlay::new(VoiceSource::Streamed(reader), AudioBus::Music)).expect("a voice is free");

  engine.set_bus_gain(AudioBus::Music, 0.5).expect("the audio thread takes commands");
  println!("{:?}", engine.get_playback(voice));

  Ok(())
}
```

`AudioEngine::play` answers an `AudioVoiceId`. Pass it to `stop`, `pause`, `resume`, `set_voice_gain`,
`set_voice_rate` and `get_playback`.

- A call for a voice whose slot has since played another sound is ignored, so a late call never reaches the wrong voice.
- `get_playback` reports the state and the source frame the audio thread last published, once per buffer. Before the
  audio thread takes a voice up, it reports the frame the voice starts at.

## Pick a layer

| Need                                                 | Use                                                     |
| ---------------------------------------------------- | ------------------------------------------------------- |
| Play voices from your own control loop               | `AudioEngine` with an `AudioOutput`                     |
| Render the mixer without a device                    | `OfflineRender`                                         |
| Render a whole scene repeatably, without a device    | `OfflineSession`                                        |
| Media-player controls for one sound                  | `AudioTransport`                                        |
| Steer audio from any thread and receive state events | `AudioServer`, which owns the engine and its transports |
| Play a level's emitters with placement and acoustics | `AudioServer::open_scene`                               |

## The audio thread

`AudioMixer::render` takes no lock and allocates nothing; a test renders every command under `assert_no_alloc`. The
control side builds and boxes each voice, the mixer moves it into one of its preallocated slots, and a finished voice
returns through a ring to be freed by the engine on its next call (or `AudioEngine::collect`).

The control side reaches the audio thread two ways:

| Way                    | Carries                                              | When it is full                                 |
| ---------------------- | ---------------------------------------------------- | ----------------------------------------------- |
| Command ring           | A voice's start, pause, resume, stop; bus and master | Refused with `AudioRefusal::CommandsFull`       |
| Slot's parameter block | Gain, rate, pan, and a placed voice's spatial block  | Never: the latest write wins, read once a block |

Voice controls never queue, because a level's emitters change them every frame and a device may not be taking buffers
at all: a thousand voices updated every frame cost a thousand writes and nothing else. A full voice pool refuses a voice
with `AudioRefusal::VoicesExhausted`.

### Blocks and timing

- The mixer renders in blocks of `AudioMixer::BLOCK_FRAMES` frames and takes commands and parameters before each, so a
  command waits at most one block whatever the device's buffer.
- `AudioPlay::at` names the mixer frame a voice's first frame plays at. A frame still ahead waits in silence, inside its
  block; a frame already passed plays from as far on as the voice would have got.
- `AudioMixerMeter` (from `AudioEngine::get_meter`) reads the frames rendered (the mixer's clock) and its load: a
  buffer's render time over the time it plays for, the peak since the last read, overruns, and the voices held.

Each block renders a voice in two steps:

1. **Render** its source into a block of the source's own channels, resampled to the output rate times its playback
   rate with a four-point Hermite curve. A mono sound stays mono here.
2. **Output** that block to its bus at its gain and fade; a mono block is heard alike in both channels. 3D placement,
   occlusion filtering and the reverb send belong to this step and leave the first alone.

The four buses (`Music`, `Ambience`, `World`, `Previews`) are scaled and summed under the master. Gains move over 10 ms
and a pause, resume or stop fades over 5 ms, so nothing clicks. A device with more than two channels gets silence past
the first two; a mono device gets the two averaged.

### Record the mix

`AudioServer::open_tap` hands a recorder the mix before the master, so the app's volume stays out of the recording:

- Whole blocks arrive, each with its first frame, through a ring the control side makes.
- A block the reader has no room for is dropped and counted, never waited for.
- The stream ends with `AudioTapEnd::RateChanged` when another device opens at another rate.
- `AudioMixerMeter::get_anchor` says when a frame plays, to line sound up with the picture: each device buffer anchors
  it with the device's latency, and without a device the render pace does.

## Sounds

`SoundDecoder` reads Ogg/Vorbis, as the game ships sounds, through `xrf-sound`'s `VorbisDecoder` (its own Ogg reading,
symphonia's Vorbis codec), and 16-bit WAV, as the SDK keeps sources, through symphonia.

- Only one or two channels are accepted, as the engines play nothing else.
- An Ogg/Vorbis sound's length is what `ov_pcm_total` says, the engine's count.
- Seeking is exact to the frame: the decoder lands on the packet holding the frame, or before it, and drops what it
  decodes before the frame.

A sound is held one of three ways:

- `DecodedSound` holds it whole, interleaved at its own rate; most voices play short sounds this way.
- `SoundStream::start` takes the sound's `SoundFormat` (rate, channels, length, from `SoundDecoder::get_format` or a
  header) and returns at once. Its thread opens and seeks the sound, then decodes about 0.7 s ahead into a ring the
  voice reads; the voice plays silence until the first frames come. A sound that cannot be opened, does not match the
  format, or cannot be sought ends the voice, and `StreamControl::get_failure` says why. The thread ends when its voice
  is freed.
- `SoundPeaks::read` streams a sound into the loudest sample of each of a number of stretches, for a waveform, without
  holding it.

## The device

`AudioOutput` plays the mixer over cpal from a thread of its own, at the device's default rate and channels.

- It follows the system default, or the device `AudioOutputChoice::Device` names while that device is present, and
  opens a new stream when the device goes. The same mixer, and so the same voices, carry over.
- With no device it renders the mixer at a device's pace and drops the result, so commands are taken, voices play out
  and the clock runs on. It retries a device every 2 s; `AudioOutput::get_status` says why none plays.
- On macOS it does not list devices during playback and relies on the stream's own rerouting and loss errors.

`AudioDevices::list` names the system's output devices.

## A player's transport

`AudioTransport` plays one sound the way a media element does:

| Call         | Effect                                                                     |
| ------------ | -------------------------------------------------------------------------- |
| `open`       | Probes the bytes and refuses what is not a sound                           |
| `play`       | Streams from where it holds, or from the start once it has `Ended`         |
| `pause`      | Ends the voice and remembers the frame                                     |
| `seek`       | Ends the voice and remembers the frame; while playing, plays on from there |
| `stop`       | Goes back to the start                                                     |
| `set_looped` | Reaches the playing stream                                                 |
| `describe`   | Answers an `AudioTransportState`, first taking in a voice that ended since |

`AudioTransportState` carries the playback, position and duration (milliseconds on the wire), looping, and why it
failed. Every call takes the engine, so transports live where the engine does: on the control thread.

## The control thread

`AudioServer` sits between a host and the mixer. `AudioServer::start` makes the engine, starts its `AudioOutput`, and
starts a thread, `xrf-audio-control`, that owns the `AudioEngine` and every transport. Its clones are handles to that
thread, usable from any thread.

- `open_transport` probes the sound on the caller's thread and hands the transport over.
- `steer_transport` (an `AudioTransportCommand`) and `describe_transport` wait for the thread's answer, two seconds at
  most. `set_gains` (`AudioGains`: the master and each bus) and `close_transport` return at once.
- Every 10 ms the thread frees the voices the mixer is done with and sends each transport's changed state to the host's
  `AudioEventSink`: playback, looping or failure at once, the position every `AudioServer::POSITION_INTERVAL` while it
  plays. `resend_transport` sends it again for a new watcher.
- The device and the mixer's meter are read without the thread.
- `AudioServer::start_on` runs a server over an engine whose mixer the caller renders, as tests do. `shut_down` closes
  the transports and ends the thread.

The sink must hold no server handle, or the thread would keep itself alive.

## Scenes and acoustics

`AudioServer::open_scene` opens a level's sound on the control thread. The host sends `AudioSceneFrame`s (the emitters
playing, the listener, the clock) through the returned `AudioSceneInbox`, never waiting. These types come from
[xrf-audio-scene](../xrf-audio-scene/README.md) and are re-exported here, so a host that only schedules sound need not
depend on the device.

The thread plays the loudest emitters by priority, up to the engine's voice targets, each placed where the scene's
clock says and stamped to the mixer frame it starts on.

With the `steam-audio` feature, `AudioAcoustics::load` brings in Steam Audio: a placed mono voice is heard through its
distance, air absorption, the walls `AudioSceneGeometry` builds and both ears, with the listener's reverb. Without the
feature, or when the library does not load, sound plays dry with one distance curve and panning.

`OfflineSession` runs all of this without a device or a thread, on a clock of the frames it renders, so rendering the
same frames gives the same result. `with_stepped_acoustics` also runs the acoustics simulation inline on that clock.

## Known limitation

A placed voice keeps the effects made at the mixer rate it started under. After a device opens at another rate, the
voice plays through effects tuned to the old rate until it starts over.

## Checks

Run `cargo test --locked -p xrf-audio` from the repository root. The device test opens the machine's real default
device, or reports that there is none.
