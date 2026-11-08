use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};

use crate::browse::ArchiveSubjects;
use crate::preview::AudioDescriptor;

/// Report whatever the engine would read out of a sound of the open subject, without handing over the sound.
#[cfg_attr(feature = "typescript-bindings", specta::specta)]
#[tauri::command(rename = "describe_audio")]
pub(crate) async fn archives_describe_audio(
  session_id: SessionId,
  name: String,
  execution: State<'_, ExecutionState>,
  subjects: State<'_, ArchiveSubjects>,
) -> TauriResult<SessionSnapshot<AudioDescriptor>> {
  log::info!("Describing audio: {name}");

  subjects
    .read_on_pool(&execution, session_id, "Describing audio", move |subject| {
      Ok(AudioDescriptor::of(&name, &subject.read_bytes(&name)?))
    })
    .await
}
