use serde::{Deserialize, Serialize};

/// A picture a helper draws at a point at a constant size on screen: a cell of the renderer's icon atlas
/// (`assets/helpers/icons.png`), each rasterised from the Material icon its row of [`RENDER_HELPER_ICONS`] names, as
/// the page shows the same one beside the kind.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RenderHelperIcon {
  Light,
  Sound,
  Particles,
  ActorStart,
  Stalker,
  Creature,
  Trader,
  Squad,
  SmartCover,
  PatrolPoint,
  GraphPoint,
  RespawnPoint,
}

/// One icon's row of the icon table: the icon, and the Material icon it is drawn from, by its `@mui/icons-material`
/// module name.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderHelperIconEntry {
  pub icon: RenderHelperIcon,
  pub material: &'static str,
}

/// Every helper icon in the atlas's order, row by row and left to right, with the Material icon each is drawn from,
/// which the page's kind rows show too.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub const RENDER_HELPER_ICONS: &[RenderHelperIconEntry] = &[
  RenderHelperIconEntry::new(RenderHelperIcon::Light, "Lightbulb"),
  RenderHelperIconEntry::new(RenderHelperIcon::Sound, "VolumeUp"),
  RenderHelperIconEntry::new(RenderHelperIcon::Particles, "AutoAwesome"),
  RenderHelperIconEntry::new(RenderHelperIcon::ActorStart, "Flag"),
  RenderHelperIconEntry::new(RenderHelperIcon::Stalker, "Person"),
  RenderHelperIconEntry::new(RenderHelperIcon::Creature, "Pets"),
  RenderHelperIconEntry::new(RenderHelperIcon::Trader, "Storefront"),
  RenderHelperIconEntry::new(RenderHelperIcon::Squad, "Groups"),
  RenderHelperIconEntry::new(RenderHelperIcon::SmartCover, "Shield"),
  RenderHelperIconEntry::new(RenderHelperIcon::PatrolPoint, "Place"),
  RenderHelperIconEntry::new(RenderHelperIcon::GraphPoint, "Hub"),
  RenderHelperIconEntry::new(RenderHelperIcon::RespawnPoint, "RestartAlt"),
];

impl RenderHelperIconEntry {
  const fn new(icon: RenderHelperIcon, material: &'static str) -> Self {
    Self { icon, material }
  }
}

impl RenderHelperIcon {
  /// Its cell in the atlas: its row's place in [`RENDER_HELPER_ICONS`].
  pub fn get_index(self) -> u32 {
    RENDER_HELPER_ICONS
      .iter()
      .position(|entry| entry.icon == self)
      .unwrap_or(0) as u32
  }

  /// The Material icon it is drawn from, by its `@mui/icons-material` module name.
  pub fn get_material_name(self) -> &'static str {
    RENDER_HELPER_ICONS[self.get_index() as usize].material
  }

  /// The name it serialises to, in camel case, as the manifest names it.
  pub fn get_name(self) -> &'static str {
    match self {
      Self::Light => "light",
      Self::Sound => "sound",
      Self::Particles => "particles",
      Self::ActorStart => "actorStart",
      Self::Stalker => "stalker",
      Self::Creature => "creature",
      Self::Trader => "trader",
      Self::Squad => "squad",
      Self::SmartCover => "smartCover",
      Self::PatrolPoint => "patrolPoint",
      Self::GraphPoint => "graphPoint",
      Self::RespawnPoint => "respawnPoint",
    }
  }

  /// The file its exported picture is kept in beside the manifest, in kebab case.
  pub fn get_picture_file(self) -> String {
    let mut file: String = String::new();

    for letter in self.get_name().chars() {
      if letter.is_ascii_uppercase() {
        file.push('-');
      }

      file.push(letter.to_ascii_lowercase());
    }

    file + ".svg"
  }
}
