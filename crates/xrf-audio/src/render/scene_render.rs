use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::control::audio_engine::AudioEngine;
use crate::control::audio_play::AudioPlay;
use crate::decode::stream::sound_stream_start::SoundStreamStart;
use crate::decode::stream::stream_control::StreamControl;
use crate::host::audio_asset_source::AudioAssetSource;
use crate::mixer::audio_bus::AudioBus;
use crate::render::emitter_loudness::{EmitterCulling, EmitterLoudness};
use crate::render::listener_pan::ListenerPan;
use crate::render::scene_clock::SceneClock;
use crate::render::sound_bank::{BankedSound, SoundBank};
use crate::render::stream_launcher::StreamLauncher;
use crate::scene::audio_emitter_report::AudioEmitterReport;
use crate::scene::audio_listener::AudioListener;
use crate::scene::audio_scene_emitter::AudioSceneEmitter;
use crate::scene::audio_scene_frame::AudioSceneFrame;
use crate::scene::audio_scene_refusal::AudioSceneRefusal;
use crate::scene::audio_scene_report::AudioSceneReport;
use crate::scene::audio_scene_rules::AudioSceneRules;
use crate::voice::audio_voice_id::AudioVoiceId;
use crate::voice::audio_voice_playback::AudioVoicePlayback;
use crate::voice::audio_voice_state::AudioVoiceState;
use crate::voice::voice_source::VoiceSource;

/// One scene as the control thread plays it, xrSound's model on the mixer: every emitter its frames name simulates by
/// the scene's clock, attenuated and smoothed as its engine does, and the loudest by priority, up to the engine's
/// targets, hold a voice placed where its clock says it is.
pub(crate) struct SceneRender {
  scene: u32,
  rules: AudioSceneRules,
  clock: SceneClock,
  listener: AudioListener,
  is_audible: bool,
  heard: [bool; AudioBus::COUNT],
  /// How much of it is heard, moving toward heard or silent over [`Self::FADE`].
  presence: f32,
  emitters: HashMap<u64, RenderedEmitter>,
  /// The scene's time at the last tick.
  ticked_at: Option<Duration>,
  refused: Vec<AudioSceneRefusal>,
  /// Candidates for a target at the last tick, loudest first.
  ranked: Vec<(f32, u64)>,
  real_voices: u32,
}

struct RenderedEmitter {
  emitter: AudioSceneEmitter,
  sound: BankedSound,
  loudness: Option<EmitterLoudness>,
  culling: EmitterCulling,
  distance: Option<f32>,
  voice: Option<RealVoice>,
  /// Whether it ranked among the loudest at the last tick.
  is_ranked: bool,
  /// Times its voice was started over where its clock says, having starved or ended early.
  restarts: u32,
  /// Played out, faded out after a stop, or refused: nothing more plays of it.
  is_done: bool,
  /// The frame that last named it, which a frame not naming it ends.
  seen: u64,
}

/// The voice an emitter holds, its stream's control where it streams, and the mixer frame and source frame it started
/// at, which say where it should be by now at its own pace.
struct RealVoice {
  id: AudioVoiceId,
  stream: Option<StreamControl>,
  started_at: u64,
  start_frame: u64,
}

/// Where an emitter's sound is at a moment.
struct SoundPlace {
  frame: u64,
  frames: u64,
  sample_rate: u32,
}

impl SceneRender {
  /// How long a scene takes to fall silent or be heard again as the listener moves between scenes.
  const FADE: Duration = Duration::from_millis(250);

  pub(crate) fn new() -> Self {
    Self {
      scene: 0,
      rules: AudioSceneRules::default(),
      clock: SceneClock::default(),
      listener: AudioListener::default(),
      is_audible: false,
      heard: [true; AudioBus::COUNT],
      presence: 0.0,
      emitters: HashMap::new(),
      ticked_at: None,
      refused: Vec::new(),
      ranked: Vec::new(),
      real_voices: 0,
    }
  }

  pub(crate) fn get_scene(&self) -> u32 {
    self.scene
  }

  /// Takes the scene's latest frame, arriving at mixer frame `mixer_frame` of `rate`: its emitters replace the ones
  /// before, a play no longer named ending.
  pub(crate) fn take(&mut self, frame: &AudioSceneFrame, mixer_frame: u64, rate: u32, generation: u64) {
    self.scene = frame.scene;
    self.rules = frame.rules;
    self.listener = frame.listener;
    self.is_audible = frame.is_audible;
    self.heard = frame.heard;

    if self.clock.follow(frame.time, mixer_frame, rate) {
      log::debug!("Scene {}'s clock anchored again at {:?}", frame.scene, frame.time);
    }

    for emitter in &frame.emitters {
      self
        .emitters
        .entry(emitter.id)
        .and_modify(|rendered| {
          rendered.emitter.clone_from(emitter);
          rendered.seen = generation;
        })
        .or_insert_with(|| RenderedEmitter {
          emitter: emitter.clone(),
          sound: BankedSound::Loading,
          loudness: None,
          culling: EmitterCulling {
            is_audible: false,
            priority: 0.0,
          },
          distance: None,
          voice: None,
          is_ranked: false,
          restarts: 0,
          is_done: false,
          seen: generation,
        });
    }
  }

  /// Plays the scene on to mixer frame `mixer_frame` of `rate`: lets go of the plays the last frame did not name,
  /// simulates every emitter, and realizes the loudest.
  #[allow(clippy::too_many_arguments)]
  pub(crate) fn tick(
    &mut self,
    engine: &mut AudioEngine,
    bank: &mut SoundBank,
    streams: &mut StreamLauncher,
    assets: (&Arc<str>, &Arc<dyn AudioAssetSource>),
    mixer_frame: u64,
    rate: u32,
    generation: u64,
  ) {
    self.emitters.retain(|_, rendered| {
      let is_named: bool = rendered.seen == generation;

      if !is_named && let Some(voice) = rendered.voice.take() {
        let _ = engine.stop(voice.id);
      }

      is_named
    });

    let Some(now) = self.clock.get_time(mixer_frame, rate) else {
      return;
    };
    let seconds: f32 = self.ticked_at.map_or(0.0, |at| now.saturating_sub(at).as_secs_f32());

    self.ticked_at = Some(now);
    self.presence = Self::approach_presence(self.presence, self.is_audible, seconds);
    self.ranked.clear();

    for (id, rendered) in &mut self.emitters {
      if Self::simulate(
        rendered,
        self.rules,
        &self.listener,
        now,
        seconds,
        bank,
        assets,
        &mut self.refused,
      ) {
        self.ranked.push((rendered.culling.priority, *id));
      }
    }

    let targets: usize = self.rules.get_targets();

    // A scene faded out holds no voice: its emitters simulate on, and take voices again where their clocks say once it
    // is heard, so silent scenes never crowd the pool.
    if self.presence <= 0.0 {
      self.ranked.clear();
    }

    if self.ranked.len() > targets {
      self.ranked.select_nth_unstable_by(targets, |a, b| b.0.total_cmp(&a.0));
      self.ranked.truncate(targets);
    }

    for rendered in self.emitters.values_mut() {
      rendered.is_ranked = false;
    }

    for (_, id) in &self.ranked {
      if let Some(rendered) = self.emitters.get_mut(id) {
        rendered.is_ranked = true;
      }
    }

    // Taken out for the while, so each emitter is realized against the scene it belongs to.
    let mut emitters: HashMap<u64, RenderedEmitter> = std::mem::take(&mut self.emitters);

    self.real_voices = 0;

    for rendered in emitters.values_mut() {
      if rendered.is_ranked && !rendered.is_done {
        self.real_voices += u32::from(self.realize(rendered, engine, streams, now, (mixer_frame, rate)));
      } else if let Some(voice) = rendered.voice.take() {
        let _ = engine.stop(voice.id);
      }
    }

    self.emitters = emitters;
  }

  /// Ends every voice, as a scene closing does.
  pub(crate) fn close(&mut self, engine: &mut AudioEngine) {
    for rendered in self.emitters.values_mut() {
      if let Some(voice) = rendered.voice.take() {
        let _ = engine.stop(voice.id);
      }
    }

    self.emitters.clear();
  }

  pub(crate) fn report(&self) -> AudioSceneReport {
    AudioSceneReport {
      emitters: self
        .emitters
        .values()
        .map(|rendered| AudioEmitterReport {
          id: rendered.emitter.id,
          // One culled, past its range or below the cull, is silent whatever its target would be handed.
          gain: rendered
            .loudness
            .filter(|_| rendered.culling.is_audible)
            .map_or(0.0, |loudness| loudness.get_gain(&rendered.emitter, rendered.distance)),
          is_real: rendered.voice.is_some(),
          is_loading: matches!(rendered.sound, BankedSound::Loading),
          restarts: rendered.restarts,
        })
        .collect(),
      refused: self.refused.clone(),
      real_voices: self.real_voices,
      targets: self.rules.get_targets() as u32,
      mixer_load: 0.0,
    }
  }

  /// Moves an emitter on to `now`, answering whether it is a candidate for a target.
  #[allow(clippy::too_many_arguments)]
  fn simulate(
    rendered: &mut RenderedEmitter,
    rules: AudioSceneRules,
    listener: &AudioListener,
    now: Duration,
    seconds: f32,
    bank: &mut SoundBank,
    (scope, assets): (&Arc<str>, &Arc<dyn AudioAssetSource>),
    refused: &mut Vec<AudioSceneRefusal>,
  ) -> bool {
    let emitter: &AudioSceneEmitter = &rendered.emitter;

    if rendered.is_done || now < emitter.start {
      return false;
    }

    if matches!(rendered.sound, BankedSound::Loading) {
      rendered.sound = bank.get(scope, assets, &emitter.sound);
    }

    if let BankedSound::Refused(reason) = &rendered.sound {
      if !refused.iter().any(|refusal| *refusal.sound == *emitter.sound) {
        refused.push(AudioSceneRefusal {
          sound: emitter.sound.to_string(),
          reason: reason.clone(),
        });
      }

      rendered.is_done = true;

      return false;
    }

    if Self::locate(rendered, now).is_none() && !matches!(rendered.sound, BankedSound::Loading) {
      rendered.is_done = true;

      return false;
    }

    let is_stopping: bool = emitter.stop.is_some_and(|stop| now >= stop);
    let distance: Option<f32> = (!emitter.is_2d).then(|| {
      let [x, y, z] = std::array::from_fn::<f32, 3, _>(|axis| emitter.position[axis] - listener.position[axis]);

      (x * x + y * y + z * z).sqrt()
    });
    let loudness: &mut EmitterLoudness = rendered.loudness.get_or_insert_with(|| EmitterLoudness::start(emitter));

    // The scene's geometry occludes nothing yet.
    rendered.culling = loudness.update(rules, emitter, distance, 1.0, seconds, is_stopping);
    rendered.distance = distance;

    if is_stopping && loudness.is_faded() {
      rendered.is_done = true;

      return false;
    }

    rendered.culling.is_audible && !matches!(rendered.sound, BankedSound::Loading)
  }

  /// Gives an emitter a voice where its clock says its sound is, starting one where it has none, or where its voice
  /// ended or starved past [`SceneClock::DRIFT`], and sets how it is heard. Answers whether it holds one.
  fn realize(
    &mut self,
    rendered: &mut RenderedEmitter,
    engine: &mut AudioEngine,
    streams: &mut StreamLauncher,
    now: Duration,
    (mixer_frame, rate): (u64, u32),
  ) -> bool {
    let Some(place) = Self::locate(rendered, now) else {
      return false;
    };
    let frequency: f32 = rendered.emitter.frequency;
    let starved: Option<RealVoice> = rendered.voice.take_if(|voice| {
      let pace: f64 = f64::from(place.sample_rate) * f64::from(frequency) / f64::from(rate.max(1));

      Self::is_starved(
        engine.get_playback(voice.id),
        voice,
        &place,
        (pace, frequency),
        mixer_frame,
      )
    });

    if let Some(voice) = starved {
      let _ = engine.stop(voice.id);

      if let Some(failure) = voice.stream.as_ref().and_then(StreamControl::get_failure) {
        self.refuse(rendered, failure);

        return false;
      }

      rendered.restarts += 1;
    }

    let emitter: &AudioSceneEmitter = &rendered.emitter;
    let gain: f32 = rendered
      .loudness
      .map_or(0.0, |loudness| loudness.get_gain(emitter, rendered.distance))
      * self.presence
      * f32::from(u8::from(self.heard[emitter.bus.get_index()]));
    let pan: [f32; 2] = match &rendered.sound {
      BankedSound::Decoded(sound) if sound.get_channels().get_count() == 2 => [1.0, 1.0],
      BankedSound::Streamed { format, .. } if format.channels.get_count() == 2 => [1.0, 1.0],
      _ => ListenerPan::of(&self.listener, emitter.position, emitter.is_2d),
    };

    if let Some(voice) = &rendered.voice {
      engine.set_voice_gain(voice.id, gain);
      engine.set_voice_pan(voice.id, pan);
      engine.set_voice_rate(voice.id, emitter.frequency);

      return true;
    }

    let start: SoundStreamStart = SoundStreamStart {
      frame: place.frame,
      is_looped: emitter.is_looped,
    };
    let (rate, bus): (f32, AudioBus) = (emitter.frequency, emitter.bus);
    let source: Result<(VoiceSource, Option<StreamControl>), String> = match &rendered.sound {
      BankedSound::Decoded(sound) => Ok((
        VoiceSource::Decoded {
          sound: Arc::clone(sound),
          start_frame: start.frame,
          is_looped: start.is_looped,
        },
        None,
      )),
      BankedSound::Streamed { bytes, format } => streams
        .start(Arc::clone(bytes), *format, start)
        .map(|reader| {
          let control: StreamControl = reader.get_control();

          (VoiceSource::Streamed(reader), Some(control))
        })
        .map_err(|error| error.to_string()),
      BankedSound::Loading | BankedSound::Refused(_) => return false,
    };
    let (source, stream) = match source {
      Ok(source) => source,
      Err(reason) => {
        self.refuse(rendered, reason);

        return false;
      }
    };
    let play: AudioPlay = AudioPlay {
      gain,
      rate,
      pan,
      ..AudioPlay::new(source, bus)
    };

    match engine.play(play) {
      Ok(id) => {
        rendered.voice = Some(RealVoice {
          id,
          stream,
          started_at: mixer_frame,
          start_frame: place.frame,
        });

        true
      }
      Err(refusal) => {
        let reason: String = format!("No voice was left for it: {refusal}");

        if !self.refused.iter().any(|it| *it.sound == *rendered.emitter.sound) {
          self.refused.push(AudioSceneRefusal {
            sound: rendered.emitter.sound.to_string(),
            reason,
          });
        }

        false
      }
    }
  }

  /// Where the emitter's sound is at `now` by its start, speed and loop; none once a sound played once is over, or
  /// while it is still being read.
  fn locate(rendered: &RenderedEmitter, now: Duration) -> Option<SoundPlace> {
    let (frames, sample_rate): (u64, u32) = match &rendered.sound {
      BankedSound::Decoded(sound) => (sound.get_frames_count() as u64, sound.get_sample_rate()),
      BankedSound::Streamed { format, .. } => (format.frames_count, format.sample_rate),
      BankedSound::Loading | BankedSound::Refused(_) => return None,
    };
    let emitter: &AudioSceneEmitter = &rendered.emitter;
    let played: f64 = now.saturating_sub(emitter.start).as_secs_f64() * f64::from(emitter.frequency);
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

  /// Whether a voice has ended while its emitter plays on, or lags where its own pace since it started should have it
  /// by more than the drift allowed, as a stream its decoder has not fed: it neither renders nor advances. Measured
  /// against the voice's own start, never the scene's clock, so a clock stepped or anchored again moves no voice; a voice
  /// not yet taken up by the mixer is where it was started. `pace` is source frames a mixer frame, `frequency` the
  /// emitter's speed.
  fn is_starved(
    playback: AudioVoicePlayback,
    voice: &RealVoice,
    place: &SoundPlace,
    (pace, frequency): (f64, f32),
    mixer_frame: u64,
  ) -> bool {
    match playback.state {
      AudioVoiceState::Ended => true,
      AudioVoiceState::Starting => false,
      AudioVoiceState::Playing | AudioVoiceState::Paused => {
        let frames: i64 = place.frames.max(1) as i64;
        let due: i64 = voice.start_frame as i64 + (mixer_frame.saturating_sub(voice.started_at) as f64 * pace) as i64;
        let behind: i64 = (due - playback.frame as i64).rem_euclid(frames);
        let allowed: f64 = SceneClock::DRIFT.as_secs_f64() * f64::from(place.sample_rate) * f64::from(frequency);

        // A lag past half the sound is the voice ahead across the loop seam, not behind.
        behind < frames / 2 && behind as f64 > allowed
      }
    }
  }

  fn refuse(&mut self, rendered: &mut RenderedEmitter, reason: String) {
    rendered.is_done = true;
    rendered.voice = None;

    if !self.refused.iter().any(|it| *it.sound == *rendered.emitter.sound) {
      self.refused.push(AudioSceneRefusal {
        sound: rendered.emitter.sound.to_string(),
        reason,
      });
    }
  }

  fn approach_presence(presence: f32, is_audible: bool, seconds: f32) -> f32 {
    let step: f32 = seconds / Self::FADE.as_secs_f32();

    if is_audible {
      (presence + step).min(1.0)
    } else {
      (presence - step).max(0.0)
    }
  }
}
