# xrf-audio-scene

What a host tells `xrf-audio` of a scene's sound, with no device, decoder or acoustics behind it: an `AudioSceneFrame`
of a moment on the scene's clock, holding where it is heard from (`AudioListener`), whose game's voice rules it plays by
(`AudioSceneRules`), which of its buses (`AudioBus`) are heard, and every `AudioSceneEmitter` it plays. A host builds
frames on its own thread and hands them to the audio's control thread through an `AudioSceneInbox`, whose other end,
the `AudioSceneMailbox`, hands each back to be filled again, so a warm scene allocates nothing.

`xrf-world` fills frames as it schedules a level's sound, and `xrf-audio` plays them and re-exports these types; a crate
that only schedules sound depends on this one rather than on the audio's device and decoders.

```rust
use xrf_audio_scene::{AudioSceneFrame, AudioSceneInbox};

let (mut inbox, mut mailbox) = AudioSceneInbox::open();

assert!(inbox.send(Box::new(AudioSceneFrame::default())).is_ok());

let frame: Box<AudioSceneFrame> = mailbox.take().expect("the frame came through");

mailbox.give_back(frame);
assert!(inbox.take_spent().is_some());
```

Run `cargo test --locked -p xrf-audio-scene` from the repository root.
