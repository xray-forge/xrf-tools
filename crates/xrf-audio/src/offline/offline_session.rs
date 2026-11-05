use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::acoustics::audio_acoustics::AudioAcoustics;
use crate::acoustics::audio_spatialization::AudioSpatialization;
use crate::acoustics::scene::simulation_pace::SimulationPace;
use crate::control::engine::audio_engine_parts::AudioEngineParts;
use crate::control::engine::audio_engine_settings::AudioEngineSettings;
use crate::control::server::audio_control::AudioControl;
use crate::control::server::control_request::ControlRequest;
use crate::control::server::control_runtime::ControlRuntime;
use crate::control::server::events::audio_event::AudioEvent;
use crate::mixer::audio_mixer::AudioMixer;
use crate::mixer::meter::audio_mixer_meter::AudioMixerMeter;
use crate::offline::offline_render::OfflineRender;
use crate::render::bank::sound_bank::SoundBank;
use crate::render::stream_launcher::StreamLauncher;
use crate::scene::host::audio_asset_source::AudioAssetSource;
use crate::scene::host::audio_scene_geometry::AudioSceneGeometry;
use crate::scene::report::audio_scene_report::AudioSceneReport;
use xrf_audio_scene::AudioSceneInbox;

/// The audio's whole runtime without a device or a thread, on a clock of the frames it renders: the control side ticks
/// every [`Self::TICK`] of them however the renders are cut, sounds are read and streams decode before each tick, so a
/// render with scenes and streams comes out the same every time. What tests and captures play scenes through.
pub struct OfflineSession {
  control: AudioControl,
  render: OfflineRender,
  /// Frames rendered between control ticks.
  tick_frames: usize,
  /// Frames left before the next tick.
  until_tick: usize,
  /// Frames rendered so far: the session's clock.
  rendered: u64,
  /// The instant its clock stood at nought, which its ticks are stamped from.
  began: Instant,
}

impl OfflineSession {
  /// How often the control side ticks, as the control thread does.
  const TICK: Duration = Duration::from_millis(10);

  /// A session playing dry.
  pub fn new(settings: AudioEngineSettings, output_rate: u32) -> Self {
    Self::with_acoustics(settings, output_rate, AudioAcoustics::default())
  }

  /// A session whose placed voices are heard through `acoustics`. The acoustics' simulation still runs on its own
  /// thread in real time, so what it hears arrives a few ticks later, however fast the session renders.
  pub fn with_acoustics(settings: AudioEngineSettings, output_rate: u32, acoustics: AudioAcoustics) -> Self {
    Self::open(settings, output_rate, acoustics, SimulationPace::RealTime)
  }

  /// A session whose placed voices are heard through `acoustics`, its scenes' simulations run inline as it ticks, at
  /// the cadence they run at live but on the session's clock, so what they hear depends on the frames sent alone. A
  /// scene's walls are built as it opens.
  pub fn with_stepped_acoustics(settings: AudioEngineSettings, output_rate: u32, acoustics: AudioAcoustics) -> Self {
    Self::open(settings, output_rate, acoustics, SimulationPace::Stepped)
  }

  fn open(settings: AudioEngineSettings, output_rate: u32, acoustics: AudioAcoustics, pace: SimulationPace) -> Self {
    let AudioEngineParts { engine, mixer } = AudioEngineParts::with_acoustics(settings, acoustics);
    let render: OfflineRender = OfflineRender::new(mixer, output_rate);
    let control: AudioControl = AudioControl::new(
      engine,
      Box::new(|_: AudioEvent| {}),
      Duration::from_millis(50),
      ControlRuntime {
        bank: SoundBank::inline(),
        streams: StreamLauncher::Pumped(Vec::new()),
        tick: Self::TICK,
        pace,
      },
    );

    Self {
      control,
      tick_frames: ((Self::TICK.as_secs_f64() * f64::from(render.get_output_rate())) as usize).max(1),
      until_tick: 0,
      rendered: 0,
      began: Instant::now(),
      render,
    }
  }

  /// Opens a scene played from the frames sent through the inbox answered, its sounds read through `assets`, its walls
  /// and rooms from `geometry` where there is one.
  pub fn open_scene(
    &mut self,
    assets: Arc<dyn AudioAssetSource>,
    geometry: Option<Arc<dyn AudioSceneGeometry>>,
  ) -> AudioSceneInbox {
    let (inbox, mailbox) = AudioSceneInbox::open();

    // Only a shut-down breaks the control's flow.
    let _ = self.control.handle(ControlRequest::OpenScene {
      mailbox,
      assets,
      geometry,
    });

    inbox
  }

  /// Places sounds for headphones or speakers from the next tick.
  pub fn set_spatialization(&mut self, spatialization: AudioSpatialization) {
    let _ = self.control.handle(ControlRequest::SetSpatialization(spatialization));
  }

  pub fn get_output_rate(&self) -> u32 {
    self.render.get_output_rate()
  }

  /// Frames a scene's frame is sent ahead of its time for every sound it starts to land on its frame: a tick and the
  /// block its commands land in.
  pub fn get_lead_frames(&self) -> u64 {
    (self.tick_frames + AudioMixer::BLOCK_FRAMES) as u64
  }

  /// The next `frames` stereo frames, interleaved.
  pub fn render(&mut self, frames: usize) -> Vec<f32> {
    let mut out: Vec<f32> = vec![0.0; frames * 2];

    self.render_into(&mut out);
    out
  }

  /// Fills `out` with the next stereo frames, interleaved: the control side ticks every [`Self::TICK`] of the
  /// session's frames, the streams filling before each, wherever a render begins or ends.
  pub fn render_into(&mut self, out: &mut [f32]) {
    let mut written: usize = 0;
    let frames: usize = out.len() / 2;

    while written < frames {
      if self.until_tick == 0 {
        let now: Instant =
          self.began + Duration::from_secs_f64(self.rendered as f64 / f64::from(self.render.get_output_rate()));

        self.control.tick(now);
        self.control.get_streams().pump();
        self.until_tick = self.tick_frames;
      }

      let count: usize = self.until_tick.min(frames - written);

      self.render.render_into(&mut out[written * 2..(written + count) * 2]);
      self.until_tick -= count;
      self.rendered += count as u64;
      written += count;
    }
  }

  /// How the scene the host names `scene` plays as of the last tick.
  pub fn describe_scene(&self, scene: u32) -> Option<AudioSceneReport> {
    self.control.describe_scene(scene)
  }

  pub fn get_meter(&self) -> &AudioMixerMeter {
    self.render.get_meter()
  }
}
