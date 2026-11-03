use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_render::WorldState;
use xrf_audio::{AudioEmitterReport, AudioSceneReport, AudioServer};
use xrf_renderer::{RenderSceneId, RenderViewportId};
use xrf_world::{WorldSoundRefusal, WorldSoundReport};

/// What the level a viewport shows plays and why: each sound, where it stands and how loud it is heard, the music, the
/// ambient's channels, and every sound that could not play; none for a viewport showing no level. The world says what
/// it schedules, the audio how each is heard, asked on a blocking thread since it answers from its own.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "describe_sound")]
pub async fn levels_describe_sound(
  world: State<'_, WorldState>,
  audio: State<'_, AudioServer>,
  execution: State<'_, ExecutionState>,
  viewport: RenderViewportId,
) -> TauriResult<Option<WorldSoundReport>> {
  let Some((RenderSceneId(scene), report)) = world.lock().report_sound(viewport) else {
    return Ok(None);
  };
  let audio: AudioServer = AudioServer::clone(&audio);
  let heard: Option<AudioSceneReport> = execution
    .run_blocking("Describing the level's sound", move || {
      audio.describe_scene(scene).map_err(|error| error.to_string())
    })
    .await??;

  Ok(Some(match heard {
    Some(heard) => join(report, heard),
    None => report,
  }))
}

/// The world's report with how the audio hears each sound, and what it refused.
fn join(mut report: WorldSoundReport, heard: AudioSceneReport) -> WorldSoundReport {
  for voice in &mut report.voices {
    if let Some(emitter) = heard.emitters.iter().find(|emitter| emitter.id == voice.id) {
      let AudioEmitterReport {
        gain,
        is_real,
        is_loading,
        restarts,
        ..
      } = *emitter;

      voice.gain = Some(gain);
      voice.is_real = is_real;
      voice.is_loading = is_loading;
      voice.restarts = restarts;
    }
  }

  report
    .refused
    .extend(heard.refused.into_iter().map(|refusal| WorldSoundRefusal {
      sound: refusal.sound,
      kind: None,
      reason: refusal.reason,
    }));
  report.real_voices = heard.real_voices;
  report.targets = heard.targets;
  report.mixer_load = Some(heard.mixer_load);

  report
}
