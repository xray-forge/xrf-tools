use serde::Serialize;
use xrf_level_source::HelperColor;
use xrf_renderer::RenderHelperIcon;

use crate::scene::SceneToolKind;

/// One helped tool's row of the scene's helper table: how the Show action names, colours, pictures and switches its
/// helpers.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SceneHelperToolEntry {
  pub tool: SceneToolKind,
  /// The bit of `RenderHelperOptions::kinds` switching its helpers: the tool's place in the editor's tool order, which
  /// also leads each of its helpers' ids.
  pub kind: u32,
  /// Its helpers' colour, `#rrggbb` in sRGB.
  pub color: &'static str,
  /// The icon its objects without a shape draw; none for a tool whose every object has a shape.
  pub icon: Option<RenderHelperIcon>,
  /// Metres from the camera past which its helpers fade out.
  pub draw_distance: f32,
  /// Whether its helpers draw until switched off.
  pub is_helped_by_default: bool,
}

impl SceneHelperToolEntry {
  /// A row whose helpers draw until switched off.
  const fn new(tool: SceneToolKind, color: &'static str, icon: Option<RenderHelperIcon>, draw_distance: f32) -> Self {
    Self {
      tool,
      kind: tool.get_ordinal(),
      color,
      icon,
      draw_distance,
      is_helped_by_default: true,
    }
  }

  /// The colour as sRGB floats from 0 to 1.
  pub(crate) fn get_rgb(&self) -> [f32; 3] {
    HelperColor::parse_rgb(self.color)
  }

  /// Whether the rows of [`SCENE_HELPER_TOOLS`] rise strictly by tool place, as helper ids then do:
  /// `SceneHelpers::find_sphere` binary-searches them and would miss helpers of a table out of order.
  const fn is_ordered() -> bool {
    let mut row: usize = 1;

    while row < SCENE_HELPER_TOOLS.len() {
      if SCENE_HELPER_TOOLS[row - 1].kind >= SCENE_HELPER_TOOLS[row].kind {
        return false;
      }

      row += 1;
    }

    true
  }
}

/// Every tool whose objects the editor draws a helper for, in the editor's tool order. Each with a counterpart in the
/// level viewer's kind table looks as it does there: spawn points as its characters, ways as its patrols, shapes as its
/// restrictors, lights, sounds, particles and fog volumes as their own; glows are amber and portals pale green, which
/// nothing else is.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub(crate) const SCENE_HELPER_TOOLS: &[SceneHelperToolEntry] = &[
  SceneHelperToolEntry::new(SceneToolKind::Glow, "#ffb74d", None, 150.0),
  SceneHelperToolEntry::new(SceneToolKind::Light, "#fff3c4", Some(RenderHelperIcon::Light), 200.0),
  SceneHelperToolEntry::new(SceneToolKind::Shape, "#2196f3", None, 250.0),
  SceneHelperToolEntry::new(
    SceneToolKind::SoundSource,
    "#80deea",
    Some(RenderHelperIcon::Sound),
    150.0,
  ),
  SceneHelperToolEntry::new(
    SceneToolKind::Spawn,
    "#e040fb",
    Some(RenderHelperIcon::SpawnPoint),
    300.0,
  ),
  SceneHelperToolEntry::new(
    SceneToolKind::Way,
    "#ffeb3b",
    Some(RenderHelperIcon::PatrolPoint),
    250.0,
  ),
  SceneHelperToolEntry::new(SceneToolKind::Portal, "#aed581", None, 250.0),
  SceneHelperToolEntry::new(SceneToolKind::SoundEnvironment, "#4dd0e1", None, 150.0),
  SceneHelperToolEntry::new(
    SceneToolKind::Particle,
    "#b39ddb",
    Some(RenderHelperIcon::Particles),
    150.0,
  ),
  SceneHelperToolEntry::new(SceneToolKind::FogVolume, "#9575cd", None, 150.0),
];

const _: () = assert!(SceneHelperToolEntry::is_ordered());
