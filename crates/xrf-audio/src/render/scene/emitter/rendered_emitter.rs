use std::time::Duration;

use crate::acoustics::scene::scene_acoustics::SceneAcoustics;
use crate::acoustics::scene::source_acoustics::SourceAcoustics;
use crate::render::bank::banked_sound::BankedSound;
use crate::render::bank::sound_bank::SoundBank;
use crate::render::emitter_culling::EmitterCulling;
use crate::render::emitter_loudness::EmitterLoudness;
use crate::render::scene::emitter::real_voice::RealVoice;
use crate::render::scene::emitter::sound_place::SoundPlace;
use crate::render::scene::scene_assets::SceneAssets;
use crate::render::scene::scene_moment::SceneMoment;
use crate::scene::audio_listener::AudioListener;
use crate::scene::audio_scene_emitter::AudioSceneEmitter;
use crate::scene::report::audio_scene_refusal::AudioSceneRefusal;
use crate::voice::shared::voice_spatial_parameters::VoiceSpatialParameters;

/// An emitter as its scene plays it: its sound, how loud it is and where it moves, and the voice it holds.
pub(crate) struct RenderedEmitter {
  pub(crate) emitter: AudioSceneEmitter,
  pub(crate) sound: BankedSound,
  pub(crate) loudness: Option<EmitterLoudness>,
  pub(crate) culling: EmitterCulling,
  pub(crate) distance: Option<f32>,
  /// Where it was when it last moved.
  motion: Option<EmitterMotion>,
  /// How fast it moves, metres a second.
  pub(crate) velocity: [f32; 3],
  pub(crate) voice: Option<RealVoice>,
  /// Whether it ranked among the loudest at the last tick.
  pub(crate) is_ranked: bool,
  /// How the walls shape its way to the listener as of the last tick.
  pub(crate) acoustics: SourceAcoustics,
  /// How the acoustics hear it as of the last tick, for the report; none where it plays dry.
  pub(crate) spatial: Option<VoiceSpatialParameters>,
  /// Times its voice was started over where its clock says, having starved or ended early.
  pub(crate) restarts: u32,
  /// Played out, faded out after a stop, or refused: nothing more plays of it.
  pub(crate) is_done: bool,
  /// The frame that last named it, which a frame not naming it ends.
  pub(crate) seen: u64,
}

/// Where an emitter stood, and when on the scene's clock.
#[derive(Clone, Copy)]
struct EmitterMotion {
  position: [f32; 3],
  at: Duration,
}

impl RenderedEmitter {
  /// Seconds an emitter's followed speed takes to settle about two thirds of the way to a new step's.
  const MOTION_EASE: f32 = 0.15;

  /// `emitter` as the frame `generation` first names it, its sound not yet asked for.
  pub(crate) fn new(emitter: AudioSceneEmitter, generation: u64) -> Self {
    Self {
      emitter,
      sound: BankedSound::Loading,
      loudness: None,
      culling: EmitterCulling {
        is_audible: false,
        priority: 0.0,
      },
      distance: None,
      motion: None,
      velocity: [0.0; 3],
      voice: None,
      is_ranked: false,
      acoustics: SourceAcoustics::default(),
      spatial: None,
      restarts: 0,
      is_done: false,
      seen: generation,
    }
  }

  /// Whether the acoustics place its sound: a mono sound at a place in the scene, while they run. A stereo file plays
  /// as it was recorded, as both engines play it.
  pub(crate) fn is_placeable(&self, acoustics: &SceneAcoustics) -> bool {
    acoustics.is_running() && !self.emitter.is_2d && self.sound.get_channel_count() == Some(1)
  }

  /// Moves it on to the moment's time, its sound asked of `bank` through `assets` and a refused one noted in `refused`,
  /// answering whether it is a candidate for a target: one starting within its horizon is, so its voice waits for its
  /// first frame in the mixer.
  pub(crate) fn simulate(
    &mut self,
    listener: &AudioListener,
    SceneMoment { now, horizon, seconds }: SceneMoment,
    bank: &mut SoundBank,
    assets: &SceneAssets,
    refused: &mut Vec<AudioSceneRefusal>,
  ) -> bool {
    if self.is_done || horizon < self.emitter.start {
      return false;
    }

    if matches!(self.sound, BankedSound::Loading) {
      self.sound = bank.get(&assets.scope, &assets.source, &self.emitter.sound);
    }

    if let BankedSound::Refused(reason) = &self.sound {
      AudioSceneRefusal::record(refused, &self.emitter.sound, reason.clone());
      self.is_done = true;

      return false;
    }

    if self.locate(now).is_none() && !matches!(self.sound, BankedSound::Loading) {
      self.is_done = true;

      return false;
    }

    self.follow_motion(now);

    let emitter: &AudioSceneEmitter = &self.emitter;
    let is_stopping: bool = emitter.stop.is_some_and(|stop| now >= stop);
    let distance: Option<f32> = (!emitter.is_2d).then(|| {
      let [x, y, z] = std::array::from_fn::<f32, 3, _>(|axis| emitter.position[axis] - listener.position[axis]);

      (x * x + y * y + z * z).sqrt()
    });
    let loudness: &mut EmitterLoudness = self.loudness.get_or_insert_with(EmitterLoudness::start);

    self.culling = loudness.update(emitter, distance, seconds, is_stopping);
    self.distance = distance;

    if is_stopping && loudness.is_faded() {
      self.is_done = true;

      return false;
    }

    self.culling.is_audible && !matches!(self.sound, BankedSound::Loading)
  }

  /// Where its sound is at `now` by its start, offset, speed and loop; none once a sound played once is over, or while
  /// it is still being read.
  pub(crate) fn locate(&self, now: Duration) -> Option<SoundPlace> {
    let (frames, sample_rate): (u64, u32) = match &self.sound {
      BankedSound::Decoded(sound) => (sound.get_frames_count() as u64, sound.get_sample_rate()),
      BankedSound::Streamed { format, .. } => (format.frames_count, format.sample_rate),
      BankedSound::Loading | BankedSound::Refused(_) => return None,
    };
    let emitter: &AudioSceneEmitter = &self.emitter;
    let played: f64 =
      now.saturating_sub(emitter.start).as_secs_f64() * f64::from(emitter.frequency) + emitter.offset.as_secs_f64();
    let frame: u64 = (played * f64::from(sample_rate)) as u64;

    if frames == 0 {
      return None;
    }

    if emitter.is_looped {
      Some(SoundPlace {
        frame: frame % frames,
        frames,
        sample_rate,
      })
    } else {
      (frame < frames).then_some(SoundPlace {
        frame,
        frames,
        sample_rate,
      })
    }
  }

  /// Follows how fast it moves, from where it was at the last tick that moved it, eased over [`Self::MOTION_EASE`]: its
  /// place comes with the host's frames and is read at the ticks, which fall unevenly between them, so a step's own
  /// speed swings about the true one and would make the pitch waver.
  fn follow_motion(&mut self, now: Duration) {
    let position: [f32; 3] = self.emitter.position;

    let here: EmitterMotion = EmitterMotion { position, at: now };

    match self.motion {
      Some(before) if before.position == position => {
        // Still for a tenth of a second is still.
        if now.saturating_sub(before.at) > Duration::from_millis(100) {
          self.velocity = [0.0; 3];
        }
      }
      Some(before) => {
        let seconds: f32 = now.saturating_sub(before.at).as_secs_f32();

        if seconds > 0.0 {
          let share: f32 = 1.0 - (-seconds / Self::MOTION_EASE).exp();

          self.velocity = std::array::from_fn(|axis| {
            let step: f32 = (position[axis] - before.position[axis]) / seconds;

            self.velocity[axis] + (step - self.velocity[axis]) * share
          });
        }

        self.motion = Some(here);
      }
      None => self.motion = Some(here),
    }
  }
}
