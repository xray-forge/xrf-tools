use serde::Serialize;
use xrf_renderer::RenderHelperIcon;

use crate::helpers::HelperColor;
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
    HelperColor::parse_rgb(self.color)
  }

  /// What every spawned visual's row shares, its own kind, label and colour aside: drawn as its model, its helper off
  /// until asked.
  const SPAWNED: Self = Self {
    kind: LevelObjectKind::Props,
    group: LevelObjectGroup::Spawned,
    label: "",
    color: "",
    icon: None,
    draw_distance: 200.0,
    is_drawn: true,
    is_helped_by_default: false,
  };

  /// What every character's row shares, its own kind, label, colour and icon aside: no picture, its icon off until
  /// asked.
  const CHARACTER: Self = Self {
    kind: LevelObjectKind::Actor,
    group: LevelObjectGroup::Characters,
    label: "",
    color: "",
    icon: None,
    draw_distance: 300.0,
    is_drawn: false,
    is_helped_by_default: false,
  };

  /// What every logic volume's row shares, its own kind, label, colour and draw distance aside: no picture, its outline
  /// drawn once helpers are.
  const VOLUME: Self = Self {
    kind: LevelObjectKind::SmartTerrains,
    group: LevelObjectGroup::Volumes,
    label: "",
    color: "",
    icon: None,
    draw_distance: 250.0,
    is_drawn: false,
    is_helped_by_default: true,
  };

  /// What every placement's row shares, its own kind, label, colour and icon aside: no picture, its helper off until
  /// asked.
  const PLACEMENT: Self = Self {
    kind: LevelObjectKind::StaticParticles,
    group: LevelObjectGroup::Placements,
    label: "",
    color: "",
    icon: None,
    draw_distance: 150.0,
    is_drawn: false,
    is_helped_by_default: false,
  };
}

/// Every kind of level object, in the order `LevelObjectKind` declares them: smart terrains orange, restrictors blue,
/// anomalies red, level changers green, patrols yellow and graph points pale grey as the map has drawn them, the
/// spawned visuals teal, characters magenta, smart covers cyan, placements muted violet and cyan, lights warm white.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub(crate) const LEVEL_OBJECT_KINDS: &[LevelObjectKindEntry] = &[
  LevelObjectKindEntry {
    kind: LevelObjectKind::Props,
    label: "Props",
    color: "#26a69a",
    ..LevelObjectKindEntry::SPAWNED
  },
  LevelObjectKindEntry {
    kind: LevelObjectKind::Items,
    label: "Items",
    color: "#4db6ac",
    ..LevelObjectKindEntry::SPAWNED
  },
  LevelObjectKindEntry {
    kind: LevelObjectKind::Weapons,
    label: "Weapons",
    color: "#00897b",
    ..LevelObjectKindEntry::SPAWNED
  },
  LevelObjectKindEntry {
    kind: LevelObjectKind::Lamps,
    label: "Lamps",
    color: "#80cbc4",
    ..LevelObjectKindEntry::SPAWNED
  },
  LevelObjectKindEntry {
    kind: LevelObjectKind::Actor,
    label: "Actor start",
    color: "#f06292",
    icon: Some(RenderHelperIcon::ActorStart),
    ..LevelObjectKindEntry::CHARACTER
  },
  LevelObjectKindEntry {
    kind: LevelObjectKind::Stalkers,
    label: "Stalkers",
    color: "#e040fb",
    icon: Some(RenderHelperIcon::Stalker),
    ..LevelObjectKindEntry::CHARACTER
  },
  LevelObjectKindEntry {
    kind: LevelObjectKind::Creatures,
    label: "Creatures",
    color: "#ab47bc",
    icon: Some(RenderHelperIcon::Creature),
    ..LevelObjectKindEntry::CHARACTER
  },
  LevelObjectKindEntry {
    kind: LevelObjectKind::Traders,
    label: "Traders",
    color: "#ce93d8",
    icon: Some(RenderHelperIcon::Trader),
    ..LevelObjectKindEntry::CHARACTER
  },
  LevelObjectKindEntry {
    kind: LevelObjectKind::Squads,
    label: "Squads",
    color: "#ba68c8",
    icon: Some(RenderHelperIcon::Squad),
    ..LevelObjectKindEntry::CHARACTER
  },
  LevelObjectKindEntry {
    kind: LevelObjectKind::SmartTerrains,
    label: "Smart terrains",
    color: "#ff9800",
    draw_distance: 600.0,
    ..LevelObjectKindEntry::VOLUME
  },
  LevelObjectKindEntry {
    kind: LevelObjectKind::Restrictors,
    label: "Restrictors",
    color: "#2196f3",
    ..LevelObjectKindEntry::VOLUME
  },
  LevelObjectKindEntry {
    kind: LevelObjectKind::Anomalies,
    label: "Anomalies",
    color: "#f44336",
    ..LevelObjectKindEntry::VOLUME
  },
  LevelObjectKindEntry {
    kind: LevelObjectKind::LevelChangers,
    label: "Level changers",
    color: "#4caf50",
    draw_distance: 600.0,
    ..LevelObjectKindEntry::VOLUME
  },
  LevelObjectKindEntry {
    kind: LevelObjectKind::Climbables,
    label: "Ladders",
    color: "#a1887f",
    draw_distance: 150.0,
    ..LevelObjectKindEntry::VOLUME
  },
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
  LevelObjectKindEntry {
    kind: LevelObjectKind::StaticParticles,
    label: "Static particles",
    color: "#b39ddb",
    icon: Some(RenderHelperIcon::Particles),
    ..LevelObjectKindEntry::PLACEMENT
  },
  LevelObjectKindEntry {
    kind: LevelObjectKind::StaticSounds,
    label: "Static sounds",
    color: "#80deea",
    icon: Some(RenderHelperIcon::Sound),
    ..LevelObjectKindEntry::PLACEMENT
  },
  LevelObjectKindEntry {
    kind: LevelObjectKind::SoundEnvironments,
    label: "Sound environments",
    color: "#4dd0e1",
    ..LevelObjectKindEntry::PLACEMENT
  },
  LevelObjectKindEntry {
    kind: LevelObjectKind::FogVolumes,
    label: "Fog volumes",
    color: "#9575cd",
    ..LevelObjectKindEntry::PLACEMENT
  },
  LevelObjectKindEntry {
    kind: LevelObjectKind::EnvModifiers,
    label: "Weather modifiers",
    color: "#7e57c2",
    ..LevelObjectKindEntry::PLACEMENT
  },
  LevelObjectKindEntry {
    kind: LevelObjectKind::RespawnPoints,
    label: "Respawn points",
    color: "#9fa8da",
    icon: Some(RenderHelperIcon::RespawnPoint),
    ..LevelObjectKindEntry::PLACEMENT
  },
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

/// The helper kind bit, after the kind table's, that switches the sound emitter helpers: each static and zone sound's
/// spheres at its minimum and maximum distance.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub(crate) const LEVEL_SOUND_EMITTERS_HELPER_KIND: u32 = LEVEL_OBJECT_KINDS.len() as u32;

// `RenderHelperOptions::kinds` holds one bit per kind, so a kind past its width would never draw: widen the mask first.
const _: () = assert!(LEVEL_SOUND_EMITTERS_HELPER_KIND < u32::BITS);
