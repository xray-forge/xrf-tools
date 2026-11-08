use xrf_audio::{AudioAcousticsReport, AudioEmitterReport, AudioSceneReport};
use xrf_world::{WorldSoundAcousticsReport, WorldSoundRefusal, WorldSoundReport};

/// A level's sound as a viewer is told of it: what the world schedules, with how the audio hears it.
pub(crate) struct LevelSoundReport;

impl LevelSoundReport {
  /// The world's report with how the audio hears each sound, what it refused, and how its acoustics run.
  pub(crate) fn join(mut report: WorldSoundReport, heard: AudioSceneReport) -> WorldSoundReport {
    for voice in &mut report.voices {
      if let Some(emitter) = heard.emitters.iter().find(|emitter| emitter.id == voice.id) {
        let AudioEmitterReport {
          gain,
          occlusion,
          transmission,
          reverb_send,
          is_binaural,
          is_real,
          is_loading,
          restarts,
          ..
        } = *emitter;

        voice.gain = Some(gain);
        voice.occlusion = Some(occlusion);
        voice.transmission = Some(transmission);
        voice.reverb_send = Some(reverb_send);
        voice.is_binaural = is_binaural;
        voice.is_real = is_real;
        voice.is_loading = is_loading;
        voice.restarts = restarts;
      }
    }

    let AudioAcousticsReport {
      failure,
      triangles,
      geometry_failure,
      reverb_times,
      build_ms,
      direct_ms,
      reflections_ms,
    } = heard.acoustics;

    report.acoustics = Some(WorldSoundAcousticsReport {
      failure,
      triangles,
      geometry_failure,
      reverb_times,
      build_ms,
      direct_ms,
      reflections_ms,
    });

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
}
