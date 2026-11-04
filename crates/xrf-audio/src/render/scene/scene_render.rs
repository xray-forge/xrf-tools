use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::acoustics::audio_acoustics::AudioAcoustics;
use crate::acoustics::audio_spatialization::AudioSpatialization;
use crate::acoustics::listener_space::ListenerSpace;
use crate::acoustics::scene::scene_acoustics::SceneAcoustics;
use crate::acoustics::scene::simulated_source::SimulatedSource;
use crate::acoustics::scene::simulation_pace::SimulationPace;
use crate::acoustics::scene::source_acoustics::SourceAcoustics;
use crate::control::audio_engine::AudioEngine;
use crate::control::audio_play::AudioPlay;
use crate::decode::stream::sound_stream_start::SoundStreamStart;
use crate::decode::stream::stream_control::StreamControl;
use crate::host::audio_scene_geometry::AudioSceneGeometry;
use crate::mixer::audio_bus::AudioBus;
use crate::render::bank::banked_sound::BankedSound;
use crate::render::emitter_loudness::EmitterLoudness;
use crate::render::listener_pan::ListenerPan;
use crate::render::scene::emitter::real_voice::RealVoice;
use crate::render::scene::emitter::rendered_emitter::RenderedEmitter;
use crate::render::scene::emitter::sound_place::SoundPlace;
use crate::render::scene::scene_assets::SceneAssets;
use crate::render::scene::scene_clock::SceneClock;
use crate::render::scene::scene_moment::SceneMoment;
use crate::render::scene::scene_tick::SceneTick;
use crate::scene::audio_listener::AudioListener;
use crate::scene::audio_scene_emitter::AudioSceneEmitter;
use crate::scene::audio_scene_frame::AudioSceneFrame;
use crate::scene::audio_scene_rules::AudioSceneRules;
use crate::scene::report::audio_emitter_report::AudioEmitterReport;
use crate::scene::report::audio_scene_refusal::AudioSceneRefusal;
use crate::scene::report::audio_scene_report::AudioSceneReport;
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

/// A candidate for a target, and how much it deserves one.
struct RankedEmitter {
  priority: f32,
  id: u64,
}

impl RankedEmitter {
  /// The more deserving first, equals by their ids: the emitters are kept in no order, and the ranking decides which
  /// voices start in which order, so a scene played twice from the same frames comes out the same.
  fn compare(a: &Self, b: &Self) -> std::cmp::Ordering {
    b.priority.total_cmp(&a.priority).then(a.id.cmp(&b.id))
  }
}

impl SceneRender {
  /// How long a scene takes to fall silent or be heard again as the listener moves between scenes.
  const FADE: Duration = Duration::from_millis(250);

  /// The placed voices heard through the HRTF on headphones, the loudest first; the rest are panned, which costs a
  /// fifth as much.
  const BINAURAL_VOICES: usize = 64;

  /// How much of a placed sound the listener's reverb hears.
  const REVERB_SEND: f32 = 0.5;

  /// A scene heard through `acoustics`, its walls where `geometry` has them, simulated at `pace`.
  pub(crate) fn new(
    acoustics: &AudioAcoustics,
    geometry: Option<Arc<dyn AudioSceneGeometry>>,
    pace: SimulationPace,
  ) -> Self {
    Self {
      scene: 0,
      rules: AudioSceneRules::default(),
      clock: SceneClock::default(),
      listener: AudioListener::default(),
      is_audible: false,
      heard: [true; AudioBus::COUNT],
      presence: 0.0,
      emitters: HashMap::new(),
      acoustics: SceneAcoustics::open(acoustics, geometry, pace),
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
        .or_insert_with(|| RenderedEmitter::new(emitter.clone(), generation));
    }
  }

  /// Plays the scene on to the tick's mixer frame, its sounds read through `assets`: lets go of the plays the last
  /// frame did not name, simulates every emitter, realizes the loudest, and hands the acoustics where the placed ones
  /// are.
  pub(crate) fn tick(&mut self, mut tick: SceneTick<'_>, assets: &SceneAssets, generation: u64) {
    self.emitters.retain(|_, rendered| {
      let is_named: bool = rendered.seen == generation;

      if !is_named && let Some(voice) = rendered.voice.take() {
        tick.engine.let_go(voice.id);
      }

      is_named
    });

    let Some(now) = self.clock.get_time(tick.mixer_frame, tick.rate) else {
      return;
    };
    let seconds: f32 = self.ticked_at.map_or(0.0, |at| now.saturating_sub(at).as_secs_f32());
    let moment: SceneMoment = SceneMoment {
      now,
      horizon: now + tick.lookahead,
      seconds,
    };

    self.ticked_at = Some(now);
    self.presence = self.approach_presence(seconds);
    self.ranked.clear();

    for (id, rendered) in &mut self.emitters {
      if rendered.simulate(&self.listener, moment, tick.bank, assets, &mut self.refused) {
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
      self.ranked.select_nth_unstable_by(targets, RankedEmitter::compare);
      self.ranked.truncate(targets);
    }

    self.ranked.sort_unstable_by(RankedEmitter::compare);

    for rendered in self.emitters.values_mut() {
      rendered.is_ranked = false;
    }

    self.simulated.clear();

    for ranked in &self.ranked {
      if let Some(rendered) = self.emitters.get_mut(&ranked.id) {
        rendered.is_ranked = true;

        if rendered.is_placeable(&self.acoustics) {
          self.simulated.push(SimulatedSource {
            id: ranked.id,
            position: rendered.emitter.position,
          });
        }
      }
    }

    if self.acoustics.is_running() {
      self.acoustics.simulate(&self.listener, &self.simulated, now);

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

      let ears: AudioSpatialization = if binaural_left > 0 && rendered.is_placeable(&self.acoustics) {
        AudioSpatialization::Headphones
      } else {
        AudioSpatialization::Speakers
      };

      if self.realize(rendered, &mut tick, now, seconds, ears) {
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
          tick.engine.let_go(voice.id);
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
        engine.let_go(voice.id);
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

  /// Gives an emitter a voice where its clock says its sound is at `now`, starting one where it has none, or where its
  /// voice ended or starved past [`SceneClock::DRIFT`], and sets how it is heard, through the HRTF or panned as `ears`
  /// says where the acoustics place it, the walls' hold eased over the `seconds` since the last tick. Answers whether it
  /// holds one.
  fn realize(
    &mut self,
    rendered: &mut RenderedEmitter,
    tick: &mut SceneTick<'_>,
    now: Duration,
    seconds: f32,
    ears: AudioSpatialization,
  ) -> bool {
    let Some(place) = rendered.locate(now) else {
      return false;
    };
    let frequency: f32 = rendered.emitter.frequency;
    let starved: Option<RealVoice> = rendered
      .voice
      .take_if(|voice| Self::is_starved(tick.engine.get_playback(voice.id), voice, &place, tick));
    let engine: &mut AudioEngine = &mut *tick.engine;

    if let Some(voice) = starved {
      engine.let_go(voice.id);

      if let Some(failure) = voice.stream.as_ref().and_then(StreamControl::get_failure) {
        self.refuse(rendered, failure);

        return false;
      }

      rendered.restarts += 1;
    }

    // A voice keeps how it started: one whose effects could not be made plays dry to its end.
    let mut is_placed: bool = match &rendered.voice {
      Some(voice) => voice.is_placed,
      None => rendered.is_placeable(&self.acoustics),
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

    // A voice starting takes the walls as they stand; one playing eases toward each new simulation.
    rendered.acoustics = match (is_placed, rendered.voice.is_some()) {
      (true, true) => {
        let mut acoustics: SourceAcoustics = rendered.acoustics;

        acoustics.approach(self.acoustics.get_source(emitter.id), seconds);
        acoustics
      }
      (true, false) => self.acoustics.get_source(emitter.id),
      (false, _) => SourceAcoustics::default(),
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
    let gain: f32 = if is_placed { scale } else { scale * attenuation };
    let pan: [f32; 2] = if is_placed || rendered.sound.get_channel_count() == Some(2) {
      [1.0, 1.0]
    } else {
      ListenerPan::of(&self.listener, emitter.position, emitter.is_2d)
    };

    if let Some(voice) = &mut rendered.voice {
      engine.set_voice_gain(voice.id, gain);
      engine.set_voice_pan(voice.id, pan);
      engine.set_voice_rate(voice.id, rate);
      voice.reckon(tick.mixer_frame, place.sample_rate, tick.rate, rate);

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
    let source: Result<VoiceSource, String> = match &rendered.sound {
      BankedSound::Decoded(sound) => Ok(VoiceSource::Decoded {
        sound: Arc::clone(sound),
        start_frame: start.frame,
        is_looped: start.is_looped,
      }),
      BankedSound::Streamed { bytes, format } => tick
        .streams
        .start(Arc::clone(bytes), *format, start)
        .map(VoiceSource::Streamed)
        .map_err(|error| error.to_string()),
      BankedSound::Loading | BankedSound::Refused(_) => return false,
    };
    let source: VoiceSource = match source {
      Ok(source) => source,
      Err(reason) => {
        self.refuse(rendered, reason);

        return false;
      }
    };
    let stream: Option<StreamControl> = source.get_stream_control();
    // Its first frame plays at the mixer frame its clock puts it at: where it stands now, or where it starts ahead.
    let at: u64 = self
      .clock
      .get_frame(now.max(rendered.emitter.start), tick.rate)
      .unwrap_or(tick.mixer_frame);
    let play: AudioPlay = AudioPlay {
      gain,
      rate,
      pan,
      at: Some(at),
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
          due: place.frame as f64,
          reckoned_at: at,
          rate,
          is_placed,
        });

        true
      }
      Err(refusal) => {
        AudioSceneRefusal::record(
          &mut self.refused,
          &rendered.emitter.sound,
          format!("No voice was left for it: {refusal}"),
        );

        false
      }
    }
  }

  /// Whether a voice has ended while its emitter plays on, or lags where its own reckoning should have it by more than
  /// the drift allowed, as a stream its decoder has not fed: it neither renders nor advances. Measured against the
  /// voice's own start at the rates it was set to, never the scene's clock, so neither a clock stepped or anchored again
  /// nor a Doppler shift moves a voice; a voice not yet taken up by the mixer is where it was started.
  fn is_starved(playback: AudioVoicePlayback, voice: &RealVoice, place: &SoundPlace, tick: &SceneTick<'_>) -> bool {
    match playback.state {
      AudioVoiceState::Ended => true,
      AudioVoiceState::Starting => false,
      AudioVoiceState::Playing | AudioVoiceState::Paused => {
        let frames: i64 = place.frames.max(1) as i64;
        let due: i64 = voice.get_due(tick.mixer_frame, place.sample_rate, tick.rate) as i64;
        let behind: i64 = (due - playback.frame as i64).rem_euclid(frames);
        let allowed: f64 = SceneClock::DRIFT.as_secs_f64() * f64::from(place.sample_rate) * f64::from(voice.rate);

        // A lag past half the sound is the voice ahead across the loop seam, not behind.
        behind < frames / 2 && behind as f64 > allowed
      }
    }
  }

  /// Ends an emitter for good, its sound refused for `reason`.
  fn refuse(&mut self, rendered: &mut RenderedEmitter, reason: String) {
    rendered.is_done = true;
    rendered.voice = None;
    AudioSceneRefusal::record(&mut self.refused, &rendered.emitter.sound, reason);
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
