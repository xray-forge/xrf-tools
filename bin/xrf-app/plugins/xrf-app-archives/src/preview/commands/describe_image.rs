use std::sync::Arc;

use serde::Serialize;
use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_extension::XrayExtensionOf;
use xrf_texture::ImageShape;

use crate::browse::{ArchiveSubject, ArchiveSubjects};

/// What a picture the webview renders itself turns out to be.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageDescriptor {
  /// Absent when the bytes carry no header this reads, which is a picture worth neither drawing nor measuring.
  pub shape: Option<ImageShape>,
  /// What to hand the bytes over as, taken from the extension rather than from the content.
  pub media_type: String,
}

/// Report the shape of a picture of the open subject the webview draws as it stands.
///
/// # Errors
///
/// Returns an error when the subject holds no such file, its bytes cannot be read, or its extension is not one a
/// webview draws - which the caller decided before asking, so reaching it means the two disagree.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "describe_image"))]
#[tauri::command(rename = "describe_image")]
pub async fn archives_describe_image(
  session_id: SessionId,
  name: String,
  execution: State<'_, ExecutionState>,
  subjects: State<'_, ArchiveSubjects>,
) -> TauriResult<ImageDescriptor> {
  log::info!("Describing image: {name}");

  let media_type: &'static str = XrayExtensionOf::of(&name)
    .known()
    .and_then(|extension| extension.get_media_type())
    .ok_or_else(|| format!("Failed to describe image '{name}': no webview renders this extension"))?;
  let subject: Arc<SessionSnapshot<ArchiveSubject>> = subjects.require(session_id)?;

  execution
    .run_blocking("Describing image", move || {
      let shape: Option<ImageShape> = ImageShape::of_bytes(&subject.read_bytes(&name)?);

      if shape.is_none() {
        log::warn!("Image '{name}' carries no header this reads");
      }

      Ok::<_, String>(ImageDescriptor {
        shape,
        media_type: media_type.to_owned(),
      })
    })
    .await?
}
