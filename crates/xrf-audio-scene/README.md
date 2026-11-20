# xrf-audio-scene

The messages a host sends to describe a scene's sound, with no audio device, decoder or acoustics attached.

[xrf-world](../xrf-world/README.md) fills them as it schedules a level's sound; [xrf-audio](../xrf-audio/README.md)
plays them and re-exports these types. A crate that only schedules sound depends on this one and stays free of the
device and decoders.

## What a frame holds

An `AudioSceneFrame` describes one moment on the scene's clock:

| Field           | Holds                                                                      |
| --------------- | -------------------------------------------------------------------------- |
| `scene`, `time` | The host's id for the scene and the scene's clock                          |
| `rules`         | `AudioSceneRules`: vanilla (32 voices at once) or extended (1024)          |
| `is_audible`    | Whether the scene is heard; one that is not fades out and plays on, silent |
| `listener`      | `AudioListener`: where the scene is heard from                             |
| `heard`         | Which buses are heard, indexed by `AudioBus::get_index`                    |
| `emitters`      | One `AudioSceneEmitter` per sound playing                                  |

## Send frames

The host builds frames on its own thread and sends them through an `AudioSceneInbox`. The audio control thread takes
them from the other end, the `AudioSceneMailbox`, and gives each back to be filled again, so a running scene allocates
nothing.

```rust
use xrf_audio_scene::{AudioSceneFrame, AudioSceneInbox};

let (mut inbox, mut mailbox) = AudioSceneInbox::open();

assert!(inbox.send(Box::new(AudioSceneFrame::default())).is_ok());

let frame: Box<AudioSceneFrame> = mailbox.take().expect("the frame came through");

mailbox.give_back(frame);
assert!(inbox.take_spent().is_some());
```

- `send` never waits: with four frames still untaken it hands the frame back, to fill again next time.
- Dropping the inbox closes the scene and fades its voices out.

## Checks

Run `cargo test --locked -p xrf-audio-scene` from the repository root.
