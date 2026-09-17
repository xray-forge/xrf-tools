use serde::Serialize;
use xrf_db::GameMtlMaterial;

/// One game material, as the viewer reads it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveGameMtlMaterial {
  /// The library's own number, which is what a collision face stores rather than the name.
  pub id: u32,
  pub name: String,
  /// Absent where the material declares no description chunk at all.
  pub description: Option<String>,
  /// The flags it sets, named.
  pub flags: Vec<String>,
  pub friction: f32,
  pub bouncing: f32,
  /// How freely a bullet passes, where 1 is straight through.
  pub shoot_factor: f32,
  /// How freely it is walked through, where below 1 the engine slows the walker down.
  pub flotation_factor: Option<f32>,
  /// How fast standing in it costs health, which is what makes a material injurious.
  pub injurious_speed: Option<f32>,
  /// How much sound it stops, where 1 lets everything through.
  pub sound_occlusion_factor: f32,
}

impl ArchiveGameMtlMaterial {
  /// Every material of a library, in the order it numbers them.
  pub fn of_all(materials: &[GameMtlMaterial]) -> Vec<Self> {
    materials.iter().map(Self::of).collect()
  }

  /// One material, taken over what it does to whatever meets it.
  fn of(material: &GameMtlMaterial) -> Self {
    Self {
      id: material.id,
      name: material.name.clone(),
      description: material.description.clone(),
      flags: material.get_named_flags().into_iter().map(ToOwned::to_owned).collect(),
      friction: material.ph_friction,
      bouncing: material.ph_bouncing,
      shoot_factor: material.shoot_factor,
      flotation_factor: material.flotation_factor,
      injurious_speed: material.injurious_speed,
      sound_occlusion_factor: material.sound_occlusion_factor,
    }
  }
}
