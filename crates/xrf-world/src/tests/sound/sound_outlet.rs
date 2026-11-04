//! A scene's sound frames: sent at most sixty times a second, out of the frames the sink hands back.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::contract::sound::world_sound_kind::WorldSoundKind;
use crate::host::sound::sink::world_sound_frame::WorldSoundFrame;
use crate::host::sound::sink::world_sound_sink::WorldSoundSink;
use crate::sound::emitters::sound_emitters::SoundEmitters;
use crate::sound::emitters::sound_outlet::SoundOutlet;
use crate::sound::emitters::sound_play::SoundPlay;
use crate::tests::sound::found;

/// A sink handing every frame straight back, noting the address each frame's emitters were held at.
#[derive(Clone, Default)]
struct ReturningSink {
  sent: Arc<Mutex<Vec<(Duration, usize, usize)>>>,
  spent: Arc<Mutex<Vec<WorldSoundFrame>>>,
}

impl WorldSoundSink for ReturningSink {
  fn send(&mut self, frame: WorldSoundFrame) {
    self
      .sent
      .lock()
      .unwrap()
      .push((frame.time, frame.emitters.len(), frame.emitters.as_ptr() as usize));
    self.spent.lock().unwrap().push(frame);
  }

  fn take_spent(&mut self) -> Option<WorldSoundFrame> {
    self.spent.lock().unwrap().pop()
  }
}

#[test]
fn sends_at_most_sixty_frames_a_second_from_the_frames_handed_back() {
  let sink: ReturningSink = ReturningSink::default();
  let mut outlet: SoundOutlet = SoundOutlet::new(Box::new(sink.clone()));
  let mut emitters: SoundEmitters = SoundEmitters::default();
  let sound = found("ambient\\wind", 100);

  emitters.play(
    xrf_engine_target::XrayEngine::Vanilla,
    SoundPlay::new(&sound, WorldSoundKind::Static, [0.0; 3], Duration::ZERO),
  );

  // A second of frames at 240 a second.
  for tick in 0..240 {
    let now: Duration = Duration::from_nanos(tick * 1_000_000_000 / 240);

    if outlet.is_due(now) {
      outlet.send(now, |frame| emitters.fill(&mut frame.emitters, now));
    }
  }

  let sent = sink.sent.lock().unwrap();

  assert!((59..=61).contains(&sent.len()), "{}", sent.len());
  assert!(sent.iter().all(|(_, count, _)| *count == 1));

  // Once the first frame came back, every frame after it is that frame filled again.
  assert!(sent.iter().skip(1).all(|(_, _, held)| *held == sent[1].2));
}

// A sound shorter than a frame still reaches the next frame once, however short it played.
#[test]
fn carries_a_short_sound_to_the_next_frame() {
  let mut emitters: SoundEmitters = SoundEmitters::default();
  let short = crate::host::sound::world_sound::WorldSound::new(
    "ambient\\click",
    Some(crate::host::sound::world_sound_info::WorldSoundInfo {
      length: Duration::from_millis(5),
      ..Default::default()
    }),
  );

  emitters.play(
    xrf_engine_target::XrayEngine::Vanilla,
    SoundPlay::new(&short, WorldSoundKind::Thunder, [0.0; 3], Duration::from_millis(1)),
  );
  emitters.retire(Duration::from_millis(16));

  let mut first: Vec<_> = Vec::new();
  let mut second: Vec<_> = Vec::new();

  emitters.fill(&mut first, Duration::from_millis(16));
  emitters.retire(Duration::from_millis(33));
  emitters.fill(&mut second, Duration::from_millis(33));

  assert_eq!(first.len(), 1);
  assert!(second.is_empty());
}
