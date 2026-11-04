use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::acoustics::audio_acoustics::AudioAcoustics;
use crate::acoustics::audio_spatialization::AudioSpatialization;
use crate::control::audio_engine_parts::AudioEngineParts;
use crate::control::audio_engine_settings::AudioEngineSettings;
use crate::control::server::audio_control::AudioControl;
use crate::control::server::audio_event::AudioEvent;
use crate::control::server::control_request::ControlRequest;
use crate::host::audio_asset_source::AudioAssetSource;
use crate::host::audio_scene_geometry::AudioSceneGeometry;
use crate::mixer::audio_mixer_meter::AudioMixerMeter;
use crate::offline::offline_render::OfflineRender;
use crate::render::sound_bank::SoundBank;
use crate::render::stream_launcher::StreamLauncher;
use crate::scene::audio_scene_inbox::AudioSceneInbox;
use crate::scene::audio_scene_report::AudioSceneReport;

/// The audio's whole runtime without a device or a thread: the control side ticks, sounds are read, and streams decode
/// in the render call, so a render with scenes and streams comes out the same every time. What tests play scenes
/// through.
pub struct OfflineSession {
  control: AudioControl,
  render: OfflineRender,
  /// Frames rendered between control ticks.
  tick_frames: usize,
}

impl OfflineSession {
  /// How often the control side ticks, as the control thread does.
  const TICK: Duration = Duration::from_millis(10);

  /// A session playing dry.
  pub fn new(settings: AudioEngineSettings, output_rate: u32) -> Self {
    Self::with_acoustics(settings, output_rate, AudioAcoustics::default())
  }

  /// A session whose placed voices are heard through `acoustics`. The acoustics' simulation still runs on its own
  /// thread, so what it hears arrives a few ticks later.
  pub fn with_acoustics(settings: AudioEngineSettings, output_rate: u32, acoustics: AudioAcoustics) -> Self {
    let AudioEngineParts { engine, mixer } = AudioEngineParts::with_acoustics(settings, acoustics);
    let render: OfflineRender = OfflineRender::new(mixer, output_rate);
    let control: AudioControl = AudioControl::with(
      engine,
      Box::new(|_: AudioEvent| {}),
      Duration::from_millis(50),
      SoundBank::inline(),
      StreamLauncher::Pumped(Vec::new()),
    );

    Self {
      control,
      tick_frames: ((Self::TICK.as_secs_f64() * f64::from(render.get_output_rate())) as usize).max(1),
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

    self.control.handle(ControlRequest::OpenScene {
      mailbox,
      assets,
      geometry,
    });

    inbox
  }

  /// Places sounds for headphones or speakers from the next tick.
  pub fn set_spatialization(&mut self, spatialization: AudioSpatialization) {
    self.control.handle(ControlRequest::SetSpatialization(spatialization));
  }

  /// The next `frames` stereo frames, interleaved: the control side ticks every [`Self::TICK`] and the streams fill
  /// before each.
  pub fn render(&mut self, frames: usize) -> Vec<f32> {
    let mut out: Vec<f32> = vec![0.0; frames * 2];

    for chunk in out.chunks_mut(self.tick_frames * 2) {
      self.control.tick(Instant::now());
      self.control.get_streams().pump();
      self.render.render_into(chunk);
    }

    out
  }

  /// How the scene the host names `scene` plays as of the last tick.
  pub fn describe_scene(&self, scene: u32) -> Option<AudioSceneReport> {
    self.control.describe_scene(scene)
  }

  pub fn get_meter(&self) -> &AudioMixerMeter {
    self.render.get_meter()
  }
}
