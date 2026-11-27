use xrf_spawn::{AlifeObjectInherited, SpawnPlacedObjects};

/// A smart terrain as the spawn places it, what its config is read from: its name and its custom data, whose
/// `[smart_terrain]` section configures it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SmartSpawn {
  pub name: String,
  pub custom_data: String,
}

impl SmartSpawn {
  /// Every Call of Pripyat smart terrain (`se_smart_terrain`) the spawn places, in its order.
  pub fn of_spawn(placed: &SpawnPlacedObjects) -> Vec<Self> {
    placed
      .objects
      .iter()
      .filter_map(|placed| match &placed.object.inherited {
        AlifeObjectInherited::SeSmartTerrain(_) => Some(Self {
          name: placed.object.header.name.clone(),
          custom_data: placed
            .object
            .inherited
            .get_abstract()
            .map(|object| object.custom_data.clone())
            .unwrap_or_default(),
        }),
        _ => None,
      })
      .collect()
  }
}
