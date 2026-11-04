use std::sync::Arc;

use xrf_audio::{
  AudioAssetSource, AudioBus, AudioListener, AudioSceneEmitter, AudioSceneFrame, AudioSceneGeometry, AudioSceneInbox,
  AudioSceneRules,
};
use xrf_engine_target::XrayEngine;
use xrf_world::{WorldSoundCategory, WorldSoundEmitter, WorldSoundFrame, WorldSoundSink};

use crate::viewer::burst::{BurstSoundFrame, BurstSoundTee};

/// A shown level's sound on its way from the world to the audio's control thread: each world frame copied into a scene
/// frame the control thread handed back, and handed back to the world at once. The world cannot depend on the audio,
/// since the renderer it feeds depends on the world, so the levels domain joins them.
pub struct LevelSoundSink {
  inbox: AudioSceneInbox,
  tee: BurstSoundTee,
  assets: Arc<dyn AudioAssetSource>,
  geometry: Arc<dyn AudioSceneGeometry>,
  /// Scene frames to fill before the control thread has handed any back, and those the inbox refused.
  // Boxed as the inbox's rings carry them, so a frame moves between the threads by its pointer.
  #[allow(clippy::vec_box)]
  spare: Vec<Box<AudioSceneFrame>>,
  /// World frames copied, for the world to fill again.
  spent: Vec<WorldSoundFrame>,
}

impl LevelSoundSink {
  /// A sink into `inbox`, the scene's sounds read through `assets` and its walls from `geometry`, a burst's frames sent
  /// on through `tee`.
  pub fn new(
    inbox: AudioSceneInbox,
    tee: BurstSoundTee,
    assets: Arc<dyn AudioAssetSource>,
    geometry: Arc<dyn AudioSceneGeometry>,
  ) -> Self {
    Self {
      inbox,
      tee,
      assets,
      geometry,
      spare: Vec::new(),
      spent: Vec::new(),
    }
  }

  fn fill(scene: &mut AudioSceneFrame, frame: &WorldSoundFrame) {
    let listener = &frame.listener;

    scene.scene = frame.scene.0;
    scene.time = frame.time;
    scene.rules = match frame.engine {
      XrayEngine::Vanilla => AudioSceneRules::Vanilla,
      XrayEngine::Extended => AudioSceneRules::Extended,
    };
    scene.is_audible = frame.is_audible;
    scene.listener = AudioListener {
      position: listener.position,
      forward: listener.forward,
      up: listener.up,
      velocity: listener.velocity,
    };
    scene.heard = AudioBus::ALL.map(|bus| match Self::to_category(bus) {
      Some(category) => frame.toggles.is_heard(category),
      None => true,
    });
    scene.emitters.clear();
    scene.emitters.extend(frame.emitters.iter().map(Self::to_emitter));
  }

  fn to_emitter(emitter: &WorldSoundEmitter) -> AudioSceneEmitter {
    AudioSceneEmitter {
      id: emitter.id,
      sound: emitter.sound.clone(),
      bus: Self::to_bus(emitter.kind.get_category()),
      position: emitter.position,
      start: emitter.start,
      offset: emitter.offset,
      stop: emitter.stop,
      is_looped: emitter.is_looped,
      is_2d: emitter.is_2d,
      volume: emitter.volume,
      frequency: emitter.frequency,
      min_distance: emitter.min_distance,
      max_distance: emitter.max_distance,
      base_volume: emitter.base_volume,
      has_reverb: emitter.has_reverb,
    }
  }

  fn to_bus(category: WorldSoundCategory) -> AudioBus {
    match category {
      WorldSoundCategory::Music => AudioBus::Music,
      WorldSoundCategory::Ambience => AudioBus::Ambience,
      WorldSoundCategory::World => AudioBus::World,
    }
  }

  fn to_category(bus: AudioBus) -> Option<WorldSoundCategory> {
    match bus {
      AudioBus::Music => Some(WorldSoundCategory::Music),
      AudioBus::Ambience => Some(WorldSoundCategory::Ambience),
      AudioBus::World => Some(WorldSoundCategory::World),
      AudioBus::Previews => None,
    }
  }
}

impl WorldSoundSink for LevelSoundSink {
  fn send(&mut self, frame: WorldSoundFrame) {
    while let Some(spent) = self.inbox.take_spent() {
      self.spare.push(spent);
    }

    let mut scene: Box<AudioSceneFrame> = self.spare.pop().unwrap_or_default();

    Self::fill(&mut scene, &frame);

    if frame.is_burst {
      if self.tee.is_armed() {
        self.tee.send(BurstSoundFrame {
          frame: scene.clone(),
          assets: Arc::clone(&self.assets),
          geometry: Arc::clone(&self.geometry),
        });
      }

      scene.is_audible = false;
    }

    // A frame the control thread has no room for waits as a spare; the next frame carries everything again.
    if let Err(refused) = self.inbox.send(scene) {
      self.spare.push(refused);
    }

    self.spent.push(frame);
  }

  fn take_spent(&mut self) -> Option<WorldSoundFrame> {
    self.spent.pop()
  }
}
