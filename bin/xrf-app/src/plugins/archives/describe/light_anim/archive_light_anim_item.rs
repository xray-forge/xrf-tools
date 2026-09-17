use serde::Serialize;
use xrf_db::LightAnimItem;

/// One colour animation, as the viewer reads it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveLightAnimItem {
  /// The name a light, a glow or a particle effect reaches this animation by.
  pub name: String,
  pub fps: f32,
  pub frames: u32,
  /// How long it runs, absent for an animation whose rate never advances it.
  pub duration_seconds: Option<f32>,
  pub keys: usize,
}

impl ArchiveLightAnimItem {
  /// Every animation of a library, in the order it numbers them.
  pub fn of_all(items: &[LightAnimItem]) -> Vec<Self> {
    items.iter().map(Self::of).collect()
  }

  /// One animation, taken over what it holds.
  fn of(item: &LightAnimItem) -> Self {
    Self {
      name: item.name.clone(),
      fps: item.fps,
      frames: item.frame_count,
      duration_seconds: item.get_duration_seconds(),
      keys: item.keys.len(),
    }
  }
}
