use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::acoustics::audio_acoustics::AudioAcoustics;
use crate::acoustics::audio_spatialization::AudioSpatialization;
use crate::acoustics::listener_space::ListenerSpace;
use crate::acoustics::scene::scene_acoustics::SceneAcoustics;
use crate::acoustics::scene::simulated_source::SimulatedSource;
use crate::acoustics::scene::source_acoustics::SourceAcoustics;
use crate::control::audio_engine::AudioEngine;
use crate::control::audio_play::AudioPlay;
use crate::decode::stream::sound_stream_start::SoundStreamStart;
use crate::decode::stream::stream_control::StreamControl;
use crate::host::audio_scene_geometry::AudioSceneGeometry;
use crate::mixer::audio_bus::AudioBus;
use crate::render::emitter_loudness::{EmitterCulling, EmitterLoudness};
use crate::render::listener_pan::ListenerPan;
use crate::render::scene::scene_assets::SceneAssets;
use crate::render::scene::scene_clock::SceneClock;
use crate::render::scene::scene_tick::SceneTick;
use crate::render::sound_bank::{BankedSound, SoundBank};
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
use crate::voice::shared::voice_spatial_parameters::VoiceSpatialParameters;
use crate::voice::voice_source::VoiceSource;
use crate::voice::voice_spatial::VoiceSpatial;

/// One scene as the control thread plays it: every emitter its frames name simulates by the scene's clock, and the
/// loudest by priority, up to the engine's targets, hold a voice placed where its clock says it is.
///
/// A placed mono sound is heard through the acoustics where they run: the distance curve, the air, the walls in its
/// way as the scene's simulation hears them, both ears, and its share of the listener's reverb; elsewhere it plays dry,
/// by the distance curve and panning. Its pitch follows its own motion and the listener's.
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
  acoustics: SceneAcoustics,
  /// The scene's time at the last tick.
  ticked_at: Option<Duration>,
  refused: Vec<AudioSceneRefusal>,
  /// Candidates for a target at the last tick, loudest first.
  ranked: Vec<RankedEmitter>,
  /// The placed sounds handed to the simulation at the last tick.
  simulated: Vec<SimulatedSource>,
  real_voices: u32,
}

struct RenderedEmitter {
  emitter: AudioSceneEmitter,
  sound: BankedSound,
  loudness: Option<EmitterLoudness>,
  culling: EmitterCulling,
  distance: Option<f32>,
  /// Where it was when it last moved.
  motion: Option<EmitterMotion>,
  /// How fast it moves, metres a second.
  velocity: [f32; 3],
  voice: Option<RealVoice>,
  /// Whether it ranked among the loudest at the last tick.
  is_ranked: bool,
  /// How the walls shape its way to the listener as of the last tick.
  acoustics: SourceAcoustics,
  /// How the acoustics hear it as of the last tick, for the report; none where it plays dry.
  spatial: Option<VoiceSpatialParameters>,
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
  /// Whether the acoustics place it.
  is_placed: bool,
}

/// Where an emitter's sound is at a moment.
struct SoundPlace {
  frame: u64,
  frames: u64,
  sample_rate: u32,
}

/// A candidate for a target, and how much it deserves one.
struct RankedEmitter {
  priority: f32,
  id: u64,
}

/// Where an emitter stood, and when on the scene's clock.
#[derive(Clone, Copy)]
struct EmitterMotion {
  position: [f32; 3],
  at: Duration,
}

impl SceneRender {
  /// How long a scene takes to fall silent or be heard again as the listener moves between scenes.
  const FADE: Duration = Duration::from_millis(250);

  /// The placed voices heard through the HRTF on headphones, the loudest first; the rest are panned, which costs a
  /// fifth as much.
  const BINAURAL_VOICES: usize = 64;

  /// How much of a placed sound the listener's reverb hears.
  const REVERB_SEND: f32 = 0.5;

  pub(crate) fn new(acoustics: &AudioAcoustics, geometry: Option<Arc<dyn AudioSceneGeometry>>) -> Self {
    Self {
      scene: 0,
      rules: AudioSceneRules::default(),
      clock: SceneClock::default(),
      listener: AudioListener::default(),
      is_audible: false,
      heard: [true; AudioBus::COUNT],
      presence: 0.0,
      emitters: HashMap::new(),
      acoustics: SceneAcoustics::open(acoustics, geometry),
      ticked_at: None,
      refused: Vec::new(),
      ranked: Vec::new(),
      simulated: Vec::new(),
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
          motion: None,
          velocity: [0.0; 3],
          voice: None,
          is_ranked: false,
          acoustics: SourceAcoustics::default(),
          spatial: None,
          restarts: 0,
          is_done: false,
          seen: generation,
        });
    }
  }

  /// Plays the scene on to the tick's mixer frame, its sounds read through `assets`: lets go of the plays the last
  /// frame did not name, simulates every emitter, realizes the loudest, and hands the acoustics where the placed ones
  /// are.
  pub(crate) fn tick(&mut self, mut tick: SceneTick<'_>, assets: &SceneAssets, generation: u64) {
    self.emitters.retain(|_, rendered| {
      let is_named: bool = rendered.seen == generation;

      if !is_named && let Some(voice) = rendered.voice.take() {
        let _ = tick.engine.stop(voice.id);
      }

      is_named
    });

    let Some(now) = self.clock.get_time(tick.mixer_frame, tick.rate) else {
      return;
    };
    let seconds: f32 = self.ticked_at.map_or(0.0, |at| now.saturating_sub(at).as_secs_f32());

    self.ticked_at = Some(now);
    self.presence = self.approach_presence(seconds);
    self.ranked.clear();

    for (id, rendered) in &mut self.emitters {
      if Self::simulate(
        rendered,
        &self.listener,
        now,
        seconds,
        tick.bank,
        assets,
        &mut self.refused,
      ) {
        self.ranked.push(RankedEmitter {
          priority: rendered.culling.priority,
          id: *id,
        });
      }
    }

    let targets: usize = self.rules.get_targets();

    // A scene faded out holds no voice: its emitters simulate on, and take voices again where their clocks say once it
    // is heard, so silent scenes never crowd the pool.
    if self.presence <= 0.0 {
      self.ranked.clear();
    }

    if self.ranked.len() > targets {
      self
        .ranked
        .select_nth_unstable_by(targets, |a, b| b.priority.total_cmp(&a.priority));
      self.ranked.truncate(targets);
    }

    self.ranked.sort_unstable_by(|a, b| b.priority.total_cmp(&a.priority));

    for rendered in self.emitters.values_mut() {
      rendered.is_ranked = false;
    }

    self.simulated.clear();

    for ranked in &self.ranked {
      if let Some(rendered) = self.emitters.get_mut(&ranked.id) {
        rendered.is_ranked = true;

        if Self::is_placeable(rendered, &self.acoustics) {
          self.simulated.push(SimulatedSource {
            id: ranked.id,
            position: rendered.emitter.position,
          });
        }
      }
    }

    if self.acoustics.is_running() {
      self.acoustics.simulate(&self.listener, &self.simulated);

      // The mixer's one reverb is the room heard now, so a scene fading out leaves it to the one fading in.
      if self.is_audible {
        tick.engine.set_reverb_times(self.acoustics.get_reverb_times());
      }
    }

    // Taken out for the while, so each emitter is realized against the scene it belongs to.
    let mut emitters: HashMap<u64, RenderedEmitter> = std::mem::take(&mut self.emitters);
    let ranked: Vec<RankedEmitter> = std::mem::take(&mut self.ranked);
    let mut binaural_left: usize = match tick.spatialization {
      AudioSpatialization::Headphones => Self::BINAURAL_VOICES,
      AudioSpatialization::Speakers => 0,
    };

    self.real_voices = 0;

    for ranked in &ranked {
      let Some(rendered) = emitters.get_mut(&ranked.id) else {
        continue;
      };

      if rendered.is_done {
        continue;
      }

      let ears: AudioSpatialization = if binaural_left > 0 && Self::is_placeable(rendered, &self.acoustics) {
        AudioSpatialization::Headphones
      } else {
        AudioSpatialization::Speakers
      };

      if self.realize(rendered, &mut tick, now, ears) {
        self.real_voices += 1;

        if ears == AudioSpatialization::Headphones {
          binaural_left -= 1;
        }
      }
    }

    for rendered in emitters.values_mut() {
      if !rendered.is_ranked || rendered.is_done {
        rendered.spatial = None;

        if let Some(voice) = rendered.voice.take() {
          let _ = tick.engine.stop(voice.id);
        }
      }
    }

    self.emitters = emitters;
    self.ranked = ranked;
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
        .map(|rendered| {
          let spatial: VoiceSpatialParameters = rendered.spatial.unwrap_or(VoiceSpatialParameters {
            reverb_send: 0.0,
            is_binaural: false,
            ..VoiceSpatialParameters::default()
          });

          AudioEmitterReport {
            id: rendered.emitter.id,
            // One culled, past its range or below the cull, is silent whatever its voice would be handed.
            gain: rendered
              .loudness
              .filter(|_| rendered.culling.is_audible)
              .map_or(0.0, |loudness| {
                loudness.get_gain(&rendered.emitter)
                  * EmitterLoudness::get_attenuation(&rendered.emitter, rendered.distance)
              }),
            occlusion: spatial.occlusion,
            transmission: spatial.transmission,
            reverb_send: spatial.reverb_send,
            is_binaural: spatial.is_binaural && rendered.voice.is_some(),
            is_real: rendered.voice.is_some(),
            is_loading: matches!(rendered.sound, BankedSound::Loading),
            restarts: rendered.restarts,
          }
        })
        .collect(),
      refused: self.refused.clone(),
      real_voices: self.real_voices,
      targets: self.rules.get_targets() as u32,
      mixer_load: 0.0,
      acoustics: self.acoustics.report(),
    }
  }

  /// Whether the acoustics place the emitter's sound: a mono sound at a place in the scene, while they run. A stereo
  /// file plays as it was recorded, as both engines play it.
  fn is_placeable(rendered: &RenderedEmitter, acoustics: &SceneAcoustics) -> bool {
    acoustics.is_running() && !rendered.emitter.is_2d && rendered.sound.get_channel_count() == Some(1)
  }

  /// Moves an emitter on to `now`, answering whether it is a candidate for a target.
  fn simulate(
    rendered: &mut RenderedEmitter,
    listener: &AudioListener,
    now: Duration,
    seconds: f32,
    bank: &mut SoundBank,
    assets: &SceneAssets,
    refused: &mut Vec<AudioSceneRefusal>,
  ) -> bool {
    let emitter: &AudioSceneEmitter = &rendered.emitter;

    if rendered.is_done || now < emitter.start {
      return false;
    }

    if matches!(rendered.sound, BankedSound::Loading) {
      rendered.sound = bank.get(&assets.scope, &assets.source, &emitter.sound);
    }

    if let BankedSound::Refused(reason) = &rendered.sound {
      Self::record_refusal(refused, &emitter.sound, reason.clone());
      rendered.is_done = true;

      return false;
    }

    if Self::locate(rendered, now).is_none() && !matches!(rendered.sound, BankedSound::Loading) {
      rendered.is_done = true;

      return false;
    }

    Self::follow_motion(rendered, now);

    let emitter: &AudioSceneEmitter = &rendered.emitter;
    let is_stopping: bool = emitter.stop.is_some_and(|stop| now >= stop);
    let distance: Option<f32> = (!emitter.is_2d).then(|| {
      let [x, y, z] = std::array::from_fn::<f32, 3, _>(|axis| emitter.position[axis] - listener.position[axis]);

      (x * x + y * y + z * z).sqrt()
    });
    let loudness: &mut EmitterLoudness = rendered.loudness.get_or_insert_with(EmitterLoudness::start);

    rendered.culling = loudness.update(emitter, distance, seconds, is_stopping);
    rendered.distance = distance;

    if is_stopping && loudness.is_faded() {
      rendered.is_done = true;

      return false;
    }

    rendered.culling.is_audible && !matches!(rendered.sound, BankedSound::Loading)
  }

  /// Follows how fast the emitter moves, from where it was at the last tick that moved it.
  fn follow_motion(rendered: &mut RenderedEmitter, now: Duration) {
    let position: [f32; 3] = rendered.emitter.position;

    let here: EmitterMotion = EmitterMotion { position, at: now };

    match rendered.motion {
      Some(before) if before.position == position => {
        // Still for a tenth of a second is still.
        if now.saturating_sub(before.at) > Duration::from_millis(100) {
          rendered.velocity = [0.0; 3];
        }
      }
      Some(before) => {
        let seconds: f32 = now.saturating_sub(before.at).as_secs_f32();

        if seconds > 0.0 {
          rendered.velocity = std::array::from_fn(|axis| (position[axis] - before.position[axis]) / seconds);
        }

        rendered.motion = Some(here);
      }
      None => rendered.motion = Some(here),
    }
  }

  /// Gives an emitter a voice where its clock says its sound is at `now`, starting one where it has none, or where its
  /// voice ended or starved past [`SceneClock::DRIFT`], and sets how it is heard, through the HRTF or panned as `ears`
  /// says where the acoustics place it. Answers whether it holds one.
  fn realize(
    &mut self,
    rendered: &mut RenderedEmitter,
    tick: &mut SceneTick<'_>,
    now: Duration,
    ears: AudioSpatialization,
  ) -> bool {
    let Some(place) = Self::locate(rendered, now) else {
      return false;
    };
    let frequency: f32 = rendered.emitter.frequency;
    let starved: Option<RealVoice> = rendered
      .voice
      .take_if(|voice| Self::is_starved(tick.engine.get_playback(voice.id), voice, &place, frequency, tick));
    let engine: &mut AudioEngine = &mut *tick.engine;

    if let Some(voice) = starved {
      let _ = engine.stop(voice.id);

      if let Some(failure) = voice.stream.as_ref().and_then(StreamControl::get_failure) {
        self.refuse(rendered, failure);

        return false;
      }

      rendered.restarts += 1;
    }

    // A voice keeps how it started: one whose effects could not be made plays dry to its end.
    let mut is_placed: bool = match &rendered.voice {
      Some(voice) => voice.is_placed,
      None => Self::is_placeable(rendered, &self.acoustics),
    };
    // todo: A placed voice keeps the effects made at the rate the mixer ran at when it started, so a device opened at
    // another rate afterwards plays it through effects tuned to the old one until it starts over.
    let spatial: Option<Box<VoiceSpatial>> = if is_placed && rendered.voice.is_none() {
      self.acoustics.create_voice_spatial(tick.rate)
    } else {
      None
    };

    is_placed &= rendered.voice.is_some() || spatial.is_some();

    let emitter: &AudioSceneEmitter = &rendered.emitter;
    let scale: f32 = rendered.loudness.map_or(0.0, |loudness| loudness.get_gain(emitter))
      * self.presence
      * f32::from(u8::from(self.heard[emitter.bus.get_index()]));
    let attenuation: f32 = EmitterLoudness::get_attenuation(emitter, rendered.distance);
    let rate: f32 = if emitter.is_2d {
      frequency
    } else {
      frequency * ListenerSpace::get_doppler(&self.listener, emitter.position, rendered.velocity)
    };

    rendered.acoustics = if is_placed {
      self.acoustics.get_source(emitter.id)
    } else {
      SourceAcoustics::default()
    };
    rendered.spatial = is_placed.then(|| VoiceSpatialParameters {
      direction: ListenerSpace::get_direction(&self.listener, emitter.position),
      attenuation,
      air_absorption: self.acoustics.get_air_absorption(&self.listener, emitter.position),
      occlusion: rendered.acoustics.occlusion,
      transmission: rendered.acoustics.transmission,
      reverb_send: if emitter.has_reverb { Self::REVERB_SEND } else { 0.0 },
      is_binaural: ears == AudioSpatialization::Headphones,
    });

    // A placed voice takes its distance through the acoustics; a dry one in its gain, panned.
    let (gain, pan): (f32, [f32; 2]) = if is_placed {
      (scale, [1.0, 1.0])
    } else {
      let pan: [f32; 2] = if rendered.sound.get_channel_count() == Some(2) {
        [1.0, 1.0]
      } else {
        ListenerPan::of(&self.listener, emitter.position, emitter.is_2d)
      };

      (scale * attenuation, pan)
    };

    if let Some(voice) = &rendered.voice {
      engine.set_voice_gain(voice.id, gain);
      engine.set_voice_pan(voice.id, pan);
      engine.set_voice_rate(voice.id, rate);

      if let Some(spatial) = &rendered.spatial {
        engine.set_voice_spatial(voice.id, spatial);
      }

      return true;
    }

    let start: SoundStreamStart = SoundStreamStart {
      frame: place.frame,
      is_looped: emitter.is_looped,
    };
    let bus: AudioBus = emitter.bus;
    let source: Result<(VoiceSource, Option<StreamControl>), String> = match &rendered.sound {
      BankedSound::Decoded(sound) => Ok((
        VoiceSource::Decoded {
          sound: Arc::clone(sound),
          start_frame: start.frame,
          is_looped: start.is_looped,
        },
        None,
      )),
      BankedSound::Streamed { bytes, format } => tick
        .streams
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
    let started = match (spatial, &rendered.spatial) {
      (Some(spatial), Some(parameters)) => tick.engine.play_placed(play, spatial, parameters),
      _ => tick.engine.play(play),
    };

    match started {
      Ok(id) => {
        rendered.voice = Some(RealVoice {
          id,
          stream,
          started_at: tick.mixer_frame,
          start_frame: place.frame,
          is_placed,
        });

        true
      }
      Err(refusal) => {
        Self::record_refusal(
          &mut self.refused,
          &rendered.emitter.sound,
          format!("No voice was left for it: {refusal}"),
        );

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
  /// not yet taken up by the mixer is where it was started. `frequency` is the emitter's speed.
  fn is_starved(
    playback: AudioVoicePlayback,
    voice: &RealVoice,
    place: &SoundPlace,
    frequency: f32,
    tick: &SceneTick<'_>,
  ) -> bool {
    match playback.state {
      AudioVoiceState::Ended => true,
      AudioVoiceState::Starting => false,
      AudioVoiceState::Playing | AudioVoiceState::Paused => {
        // Source frames a mixer frame.
        let pace: f64 = f64::from(place.sample_rate) * f64::from(frequency) / f64::from(tick.rate.max(1));
        let frames: i64 = place.frames.max(1) as i64;
        let played: u64 = tick.mixer_frame.saturating_sub(voice.started_at);
        let due: i64 = voice.start_frame as i64 + (played as f64 * pace) as i64;
        let behind: i64 = (due - playback.frame as i64).rem_euclid(frames);
        let allowed: f64 = SceneClock::DRIFT.as_secs_f64() * f64::from(place.sample_rate) * f64::from(frequency);

        // A lag past half the sound is the voice ahead across the loop seam, not behind.
        behind < frames / 2 && behind as f64 > allowed
      }
    }
  }

  /// Ends an emitter for good, its sound refused for `reason`.
  fn refuse(&mut self, rendered: &mut RenderedEmitter, reason: String) {
    rendered.is_done = true;
    rendered.voice = None;
    Self::record_refusal(&mut self.refused, &rendered.emitter.sound, reason);
  }

  /// Notes that `sound` was refused for `reason`, once a sound however many emitters play it.
  fn record_refusal(refused: &mut Vec<AudioSceneRefusal>, sound: &str, reason: String) {
    if !refused.iter().any(|refusal| *refusal.sound == *sound) {
      refused.push(AudioSceneRefusal {
        sound: sound.to_owned(),
        reason,
      });
    }
  }

  /// How much of the scene is heard `seconds` on, moving toward heard or silent.
  fn approach_presence(&self, seconds: f32) -> f32 {
    let step: f32 = seconds / Self::FADE.as_secs_f32();

    if self.is_audible {
      (self.presence + step).min(1.0)
    } else {
      (self.presence - step).max(0.0)
    }
  }
}
