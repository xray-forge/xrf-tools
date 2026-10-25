use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_extension::XrayExtensionOf;

use crate::browse::ArchiveSubjects;
use crate::preview::AudioDescriptor;

/// Report whatever the engine would read out of a sound of the open subject, without handing over the sound.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "describe_audio"))]
#[tauri::command(rename = "describe_audio")]
pub async fn archives_describe_audio(
  session_id: SessionId,
  name: String,
  execution: State<'_, ExecutionState>,
  subjects: State<'_, ArchiveSubjects>,
) -> TauriResult<SessionSnapshot<AudioDescriptor>> {
  log::info!("Describing audio: {name}");

  let media_type: &'static str = XrayExtensionOf::of(&name)
    .known()
    .and_then(|extension| extension.get_media_type())
    .ok_or_else(|| format!("Failed to describe audio '{name}': no webview plays this extension"))?;

  subjects
    .read_on_pool(&execution, session_id, "Describing audio", move |subject| {
      Ok(AudioDescriptor::of(&name, &subject.read_bytes(&name)?, media_type))
    })
    .await
}
