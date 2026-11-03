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
  SoundStream, SoundStreamStart, VoiceSource,
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
  let reader = SoundStream::start(music, SoundStreamStart { frame: 0, is_looped: true })?;
  let voice = engine.play(AudioPlay::new(VoiceSource::Streamed(reader), AudioBus::Music)).expect("a voice is free");

  engine.set_bus_gain(AudioBus::Music, 0.5).expect("the audio thread takes commands");
  println!("{:?}", engine.get_playback(voice));

  Ok(())
}
```

`AudioEngine::play` answers an `AudioVoiceId`, which `stop`, `pause`, `resume`, `set_voice_gain`, `set_voice_rate` and
`get_playback` take. A voice whose slot has since played another sound is left alone, so a late call never reaches the
wrong one. `get_playback` reports the state and the source frame the audio thread last published, once a buffer, and
the frame a voice starts at until the audio thread takes it up.

## The audio thread

`AudioMixer::render` takes no lock and allocates nothing; a test renders every command under `assert_no_alloc`. The
control side builds and boxes each voice, the mixer moves it into one of its preallocated slots, and a voice that ends
goes back through a ring to be freed by the engine on its next call (or `AudioEngine::collect`).

The control side reaches the audio thread two ways:

| Way                    | Carries                                              | When it is full                                  |
| ---------------------- | ---------------------------------------------------- | ------------------------------------------------ |
| Command ring           | A voice's start, pause, resume, stop; bus and master | Refused with `AudioRefusal::CommandsFull`        |
| Slot's parameter block | A voice's continuous controls: gain and rate         | Never: the latest write wins, read once a buffer |

A voice's controls change every frame for a level's emitters, and a device may not be taking buffers at all, so they
never queue: a thousand voices updated every frame cost a thousand writes and nothing else. A full pool refuses a voice
with `AudioRefusal::VoicesExhausted`.

The mixer renders in blocks of `AudioMixer::BLOCK_FRAMES` frames, in two steps a voice:

1. **Render** its source, resampled to the output rate times its playback rate with a four-point Hermite curve, into a
   block of the source's own channels. A mono sound stays mono here.
2. **Output** that block to its bus at its gain and fade, a mono block heard alike in both channels. Placing a sound
   in 3D, filtering it for occlusion and sending it to reverb belong to this step, and leave the first alone.

The four buses (`Music`, `Ambience`, `World`, `Previews`) are scaled and summed under the master. Gains move over
10 ms, and a pause, resume or stop fades over 5 ms, so nothing clicks. A device with more than two channels gets
silence past the first two; a mono one gets the two averaged.

## Sounds

`SoundDecoder` reads Ogg/Vorbis, as the game ships sounds, and 16-bit WAV, as the SDK keeps sources, through
symphonia. A sound with other than one or two channels is refused, as the engines play nothing else. Seeking is exact
to the frame: the decoder lands a second early, because symphonia sometimes misplaces the first packets of the Ogg page
a seek lands on, and drops what it decodes before the frame.

- `DecodedSound` holds a sound whole, interleaved at its own rate, for the short sounds most voices play.
- `SoundStream::start` opens and seeks a sound before it returns, so a sound that cannot be read is refused at once,
  then decodes on a thread about 0.7 s ahead into a ring the voice reads. The thread ends when its voice is freed.
  `StreamControl` loops a stream and names why it failed.
- `SoundPeaks::read` decodes a sound as a stream into the loudest sample of each of a number of stretches, for a
  waveform, without holding the sound.

## The device

`AudioOutput` plays the mixer from a thread of its own over cpal at the device's default rate and channels. It follows
the system default, or the device `AudioOutputChoice::Device` names while that device is present, and opens a new
stream when the device goes; the same mixer, and so the same voices, carry over. With no device it waits silent,
retrying every 2 s, and `AudioOutput::get_status` says why. On macOS it does not list devices during playback and relies
on the stream's own rerouting and loss errors. `AudioDevices::list` names the system's output devices.

## A player's transport

`AudioTransport` is what a player plays one sound through, as a media element would: `open` probes the bytes (refusing
what is not a sound), `play` streams from where it holds (from the start once it has `Ended`), `pause` and `seek` end the
voice and remember the frame (a seek while playing plays on from there), `stop` puts it back at the start, and
`set_looped` reaches the playing stream. `describe` answers an `AudioTransportState` (playback, position and duration
in milliseconds on the wire, looping, why it failed), having taken in a voice that ended since. Every call takes the
engine, so a host holds the transports beside it.

## Not here yet

Voices play in 2D only: 3D voices, the engines' distance attenuation, occlusion and reverb are not written yet, and the
voice's resampler needs a faster inner loop before 1024 voices play at once.

Run `cargo test -p xrf-audio`. The device test opens the machine's real default device, or reports that there is none.
