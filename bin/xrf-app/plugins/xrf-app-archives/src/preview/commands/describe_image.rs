use tauri::State;
use xrf_app_core::TauriResult;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_extension::XrayExtensionOf;
use xrf_texture::ImageShape;

use crate::browse::ArchiveSubjects;
use crate::preview::ImageDescriptor;

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
) -> TauriResult<SessionSnapshot<ImageDescriptor>> {
  log::info!("Describing image: {name}");

  let media_type: &'static str = XrayExtensionOf::of(&name)
    .known()
    .and_then(|extension| extension.get_media_type())
    .ok_or_else(|| format!("Failed to describe image '{name}': no webview renders this extension"))?;

  subjects
    .read_on_pool(&execution, session_id, "Describing image", move |subject| {
      let shape: Option<ImageShape> = ImageShape::of_bytes(&subject.read_bytes(&name)?);

      if shape.is_none() {
        log::warn!("Image '{name}' carries no header this reads");
      }

      Ok(ImageDescriptor {
        shape,
        media_type: media_type.to_owned(),
      })
    })
    .await
}
