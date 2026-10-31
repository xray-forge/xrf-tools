use serde::Serialize;
use xrf_renderer::RenderHelperIcon;

use crate::objects::{LevelObjectGroup, LevelObjectKind};

/// One kind's row of the kind table: what the Outliner, the Show popover, the helpers and the map all name, colour,
/// picture and switch it by.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelObjectKindEntry {
  pub kind: LevelObjectKind,
  pub group: LevelObjectGroup,
  pub label: &'static str,
  /// Its helpers' colour, `#rrggbb` in sRGB.
  pub color: &'static str,
  /// The icon its objects' helpers draw where they stand, for a kind whose objects have no shape of their own.
  pub icon: Option<RenderHelperIcon>,
  /// Metres from the camera past which its helpers fade out.
  pub draw_distance: f32,
  /// Whether its objects have a picture of their own, which the Visible switch shows or hides.
  pub is_drawn: bool,
  /// Whether its helpers are drawn until the viewer is told otherwise.
  pub is_helped_by_default: bool,
}

impl LevelObjectKindEntry {
  /// The colour as sRGB floats from 0 to 1, a channel not written as two hex digits read as zero.
  pub fn get_rgb(&self) -> [f32; 3] {
    let channel = |at: usize| -> f32 {
      self
        .color
        .get(at..at + 2)
        .and_then(|hex| u8::from_str_radix(hex, 16).ok())
        .map_or(0.0, |value| f32::from(value) / 255.0)
    };

    [channel(1), channel(3), channel(5)]
  }

  /// A spawned visual's row: drawn as its model, its helper off until asked.
  const fn spawned(kind: LevelObjectKind, label: &'static str, color: &'static str) -> Self {
    Self {
      kind,
      group: LevelObjectGroup::Spawned,
      label,
      color,
      icon: None,
      draw_distance: 200.0,
      is_drawn: true,
      is_helped_by_default: false,
    }
  }

  /// A character's row: no picture, its icon off until asked.
  const fn character(kind: LevelObjectKind, label: &'static str, color: &'static str, icon: RenderHelperIcon) -> Self {
    Self {
      kind,
      group: LevelObjectGroup::Characters,
      label,
      color,
      icon: Some(icon),
      draw_distance: 300.0,
      is_drawn: false,
      is_helped_by_default: false,
    }
  }

  /// A logic volume's row: no picture, its outline drawn once helpers are.
  const fn volume(kind: LevelObjectKind, label: &'static str, color: &'static str, draw_distance: f32) -> Self {
    Self {
      kind,
      group: LevelObjectGroup::Volumes,
      label,
      color,
      icon: None,
      draw_distance,
      is_drawn: false,
      is_helped_by_default: true,
    }
  }

  /// A placement's row: no picture, its helper off until asked; an icon for a placement at a point.
  const fn placement(
    kind: LevelObjectKind,
    label: &'static str,
    color: &'static str,
    icon: Option<RenderHelperIcon>,
  ) -> Self {
    Self {
      kind,
      group: LevelObjectGroup::Placements,
      label,
      color,
      icon,
      draw_distance: 150.0,
      is_drawn: false,
      is_helped_by_default: false,
    }
  }
}

/// Every kind of level object, in the order `LevelObjectKind` declares them: smart terrains orange, restrictors blue,
/// anomalies red, level changers green, patrols yellow and graph points pale grey as the map has drawn them, the
/// spawned visuals teal, characters magenta, smart covers cyan, placements muted violet and cyan, lights warm white.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub const LEVEL_OBJECT_KINDS: &[LevelObjectKindEntry] = &[
  LevelObjectKindEntry::spawned(LevelObjectKind::Props, "Props", "#26a69a"),
  LevelObjectKindEntry::spawned(LevelObjectKind::Items, "Items", "#4db6ac"),
  LevelObjectKindEntry::spawned(LevelObjectKind::Weapons, "Weapons", "#00897b"),
  LevelObjectKindEntry::spawned(LevelObjectKind::Lamps, "Lamps", "#80cbc4"),
  LevelObjectKindEntry::character(
    LevelObjectKind::Actor,
    "Actor start",
    "#f06292",
    RenderHelperIcon::ActorStart,
  ),
  LevelObjectKindEntry::character(
    LevelObjectKind::Stalkers,
    "Stalkers",
    "#e040fb",
    RenderHelperIcon::Stalker,
  ),
  LevelObjectKindEntry::character(
    LevelObjectKind::Creatures,
    "Creatures",
    "#ab47bc",
    RenderHelperIcon::Creature,
  ),
  LevelObjectKindEntry::character(LevelObjectKind::Traders, "Traders", "#ce93d8", RenderHelperIcon::Trader),
  LevelObjectKindEntry::character(LevelObjectKind::Squads, "Squads", "#ba68c8", RenderHelperIcon::Squad),
  LevelObjectKindEntry::volume(LevelObjectKind::SmartTerrains, "Smart terrains", "#ff9800", 600.0),
  LevelObjectKindEntry::volume(LevelObjectKind::Restrictors, "Restrictors", "#2196f3", 250.0),
  LevelObjectKindEntry::volume(LevelObjectKind::Anomalies, "Anomalies", "#f44336", 250.0),
  LevelObjectKindEntry::volume(LevelObjectKind::LevelChangers, "Level changers", "#4caf50", 600.0),
  LevelObjectKindEntry::volume(LevelObjectKind::Climbables, "Ladders", "#a1887f", 150.0),
  LevelObjectKindEntry {
    kind: LevelObjectKind::SmartCovers,
    group: LevelObjectGroup::Covers,
    label: "Smart covers",
    color: "#00bcd4",
    icon: Some(RenderHelperIcon::SmartCover),
    draw_distance: 150.0,
    is_drawn: false,
    is_helped_by_default: false,
  },
  LevelObjectKindEntry {
    kind: LevelObjectKind::Patrols,
    group: LevelObjectGroup::Paths,
    label: "Patrols",
    color: "#ffeb3b",
    icon: Some(RenderHelperIcon::PatrolPoint),
    draw_distance: 250.0,
    is_drawn: false,
    is_helped_by_default: true,
  },
  LevelObjectKindEntry {
    kind: LevelObjectKind::GraphPoints,
    group: LevelObjectGroup::Navigation,
    label: "Graph points",
    color: "#e0e0e0",
    icon: Some(RenderHelperIcon::GraphPoint),
    draw_distance: 150.0,
    is_drawn: false,
    is_helped_by_default: false,
  },
  LevelObjectKindEntry::placement(
    LevelObjectKind::StaticParticles,
    "Static particles",
    "#b39ddb",
    Some(RenderHelperIcon::Particles),
  ),
  LevelObjectKindEntry::placement(
    LevelObjectKind::StaticSounds,
    "Static sounds",
    "#80deea",
    Some(RenderHelperIcon::Sound),
  ),
  LevelObjectKindEntry::placement(
    LevelObjectKind::SoundEnvironments,
    "Sound environments",
    "#4dd0e1",
    None,
  ),
  LevelObjectKindEntry::placement(LevelObjectKind::FogVolumes, "Fog volumes", "#9575cd", None),
  LevelObjectKindEntry::placement(LevelObjectKind::EnvModifiers, "Weather modifiers", "#7e57c2", None),
  LevelObjectKindEntry::placement(
    LevelObjectKind::RespawnPoints,
    "Respawn points",
    "#9fa8da",
    Some(RenderHelperIcon::RespawnPoint),
  ),
  LevelObjectKindEntry {
    kind: LevelObjectKind::Lights,
    group: LevelObjectGroup::Lights,
    label: "Lights",
    color: "#fff3c4",
    icon: Some(RenderHelperIcon::Light),
    draw_distance: 200.0,
    is_drawn: false,
    is_helped_by_default: false,
  },
];
