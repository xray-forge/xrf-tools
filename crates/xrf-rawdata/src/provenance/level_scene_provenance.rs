use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use xrf_error::XrfResult;

use crate::level::level_geometry_conversion::LevelGeometryCensus;
use crate::level::level_surface_inference::LevelSurfaceCensus;
use crate::provenance::sidecar::Sidecar;

/// One compiled file a scene was decompiled from.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSceneSource {
  /// Its name in the level's directory, such as `level.ps_static`.
  pub file: String,
  /// Its SHA-256, lowercase hex.
  pub sha256: String,
}

/// One file the decompile wrote, and the compiled files it came from.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSceneFile {
  /// Its path relative to the scene's main file's directory, such as `labx8/ps.part`.
  pub written: String,
  pub sources: Vec<String>,
}

/// What a decompiled scene holds that its sources do not say, which the decompile chose; a part's inferences are
/// recorded only where it holds what they were chosen for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LevelSceneInference {
  /// Objects are named as the editor names new ones, `<prefix>_NNNN`, the prefix their file's name or their tool's.
  ObjectNames,
  /// The level options are a new level's, built into and prefixed by the level's name, but for the light qualities,
  /// which the count of `build.lights`' generated sky and sun gives.
  LevelOptions,
  /// The level serves the game modes its respawn points answer to, or single player where it has none; made only where
  /// no `level.ltx` names them.
  GameTypes,
  /// The build parameters are the editor's defaults but for a weld distance of 0, for geometry already welded.
  BuildParameters,
  /// The camera stands at the origin, unturned.
  Camera,
  /// Sound sources take a new source's distances, which the editor reads again from each sound file.
  SoundSourceDistances,
  /// The wall mark tool takes a new tool's brush, and each decal the brush's size and turn.
  WallmarkBrush,
  /// Glows stand unturned with no fixed-size flag, which the compiled level does not keep.
  GlowTransformsAndFlags,
  /// Each light's brightness is the channel of its diffuse of the largest magnitude, sign kept, and its colour the rest;
  /// the compiled level keeps only their product.
  LightBrightness,
  /// A fuzzy light comes back as one light per point it was spread over.
  LightFuzzyPoints,
  /// A light only dynamic answers to `$static`; the compiled level keeps no control's name.
  LightControls,
  /// An entity's volume comes back as a shape attached to it at its position, unturned and unscaled, under a generated
  /// name; the spawn list keeps only the shapes.
  EntityShapes,
  /// The static geometry comes back as one object per compiled sector, named after it, its points in the level's space
  /// and placed at the origin; sectors take a new sector's colour, and sectors and portals are named as the editor
  /// names new ones. The compiled level merged the authored objects into each sector.
  SectorShells,
  /// Each static surface's game material is the collision face's its triangles lie on, a surface split where they lie
  /// on different ones or none; its compiler shader is one of `shaders_xrlc.xr` fitting what the level shows
  /// (collision, vertex lighting), chosen as the sidecar's `geometry.surfaces` records.
  SurfaceDress,
  /// The collision faces no render triangle lies on come back as one object, `<level>_collision`, a mesh per sector,
  /// dressed as GSC dressed its collision-only props (`prop\prop_fake_kollision`, `def_shaders\def_trans_v`,
  /// `def_shaders\def_kolizion_vertex`).
  CollisionObject,
  /// `level.hom` comes back as `<level>_hom`, a surface of `materials\occ` as GSC's `labx8` keeps its occluders, and
  /// `level.som` as `<level>_som`, flagged a sound occluder, each polygon's game material the library's of the lowest
  /// number whose sound occlusion factor is the polygon's; both dressed with `prop\prop_fake_occ`.
  Occluders,
  /// Each ladder the spawn list keeps (`climable_object`) comes back as an object, `<level>_ladder_<N>`, of its box
  /// less the back face (its front alone where the box is flat), standing at the middle of the box's lowest end and
  /// dressed as GSC dressed its ladders (`prop\prop_fake_ladder`, `def_shaders\def_trans_v`,
  /// `def_shaders\def_kolizion_vertex`), of `materials\fake_ladders` where the packet keeps no game material; its
  /// packet is no spawn point, since the Build exports it again. The authored faces are lost.
  Ladders,
  /// Each breakable part the Build wrote (`meshes\brkbl#<N>.ogf`) comes back as a static object,
  /// `<level>_breakable_<N>`, standing where its `breakable_object` placed it: its fragments' bones dropped, each surface
  /// of the game material of the fragments its faces follow, its compiler shader the editor's default. Its packet is no
  /// spawn point, since the Build exports it again, cut into new fragments at random.
  BreakableParts,
  /// Trees come back as multi-use objects, `<level>_tree_<N>`, one per set of subdivisions the compiled tree visuals
  /// draw, each in its own space and dressed as its first placement lies on the collision form, with a scene object per
  /// placement; their impostors are not kept, so a rebuild draws no tree far off as one.
  TreeModels,
  /// Each detail model comes back as the detail object of the object libraries drawing its geometry (the first by
  /// path where several do), at the level's scale range and flags and a density factor of 1; the tool takes a density
  /// of 0.3, no base texture, colour indices or snap objects, and the grid's box, and each slot's light is zeroed, as
  /// the Build zeroes it before the detail lighter lights it again.
  DetailObjects,
  /// The map's rectangle (`level.ltx`'s `bound_rect`) comes back as a level-bound shape, `level_bound_NNNN`, a box over
  /// it one metre tall around height 0; the file keeps only its x and z.
  MapBound,
}

/// The objects a scene's static geometry came back as, and what the decompile counted of it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSceneGeometry {
  /// Each object written, by its path under `$objects$` without the extension.
  pub objects: Vec<String>,
  pub census: LevelGeometryCensus,
  /// How the surfaces were dressed, where they were inferred.
  #[serde(default)]
  pub surfaces: Option<LevelSceneSurfaces>,
}

/// The surfaces of a scene's static geometry as inferred (decision 14): from what, and every compiler shader chosen with
/// why.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSceneSurfaces {
  pub libraries: LevelSceneSurfaceLibraries,
  pub census: LevelSurfaceCensus,
}

/// The libraries a scene's surfaces were dressed from.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSceneSurfaceLibraries {
  pub game_materials: LevelSceneSource,
  pub compiler_shaders: LevelSceneSource,
  /// The object library roots whose pairings broke ties, as given.
  pub object_roots: Vec<String>,
  /// Objects under them that did not read.
  pub unread_objects: usize,
}

/// What the decompile read but left out of the scene, with why.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSceneRefusal {
  /// The compiled file holding it, such as `level.spawn`.
  pub file: String,
  /// Its name there, such as an entity's.
  pub object: String,
  pub reason: String,
}

/// The root spawn a scene's entities were decompiled from (decision 10): taken for the level it holds in place of
/// `level.spawn`'s entities, with what the two disagree on and what the scene cannot carry.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSceneRootSpawn {
  /// Its path as given, such as `../../spawns/all.spawn`.
  pub file: String,
  /// Its SHA-256, lowercase hex.
  pub sha256: String,
  /// Its objects standing on the level, which the scene takes; none where it does not hold the level, whose entities
  /// then come from `level.spawn`.
  pub objects: usize,
  /// Its objects of the whole spawn that did not read, on whichever level they stand, which cannot be told.
  pub unread: usize,
  pub disagreements: Vec<LevelSceneDisagreement>,
  pub update_packets: Vec<LevelSceneUpdatePacket>,
}

/// An object the root spawn and `level.spawn` disagree on, past the fields xrAI fills in.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSceneDisagreement {
  pub object: String,
  /// What differs: only one of the two holds it, or their spawn packets differ.
  pub reason: String,
}

/// An object's update packet as the root spawn stores it, where it differs from the one a fresh entity read from its
/// spawn packet holds: the scene keeps the spawn packet alone, so this is what a root spawn rebuilt from it would lose.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSceneUpdatePacket {
  pub object: String,
  /// The packet's bytes past its length and `M_UPDATE`, lowercase hex.
  pub bytes: String,
}

/// The sidecar written beside a decompiled scene, `<name>.level.xrf.json` (decision 22): the compiled files it came
/// from, what made it and when, every file it wrote and from what, and what in it was chosen rather than read.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSceneProvenance {
  /// The level's name.
  pub level: String,
  /// The front-end that made the scene, such as `xrf-cli level decompile`, and its version.
  pub tool: String,
  pub tool_version: String,
  /// Seconds since the Unix epoch.
  pub decompiled_at: u64,
  pub sources: Vec<LevelSceneSource>,
  pub files: Vec<LevelSceneFile>,
  pub inferred: Vec<LevelSceneInference>,
  pub refused: Vec<LevelSceneRefusal>,
  /// The root spawn read, where one was given.
  pub root_spawn: Option<LevelSceneRootSpawn>,
  /// The static geometry, where the level's directory held `level.geom`.
  #[serde(default)]
  pub geometry: Option<LevelSceneGeometry>,
}

impl LevelSceneProvenance {
  /// What the sidecar's name adds to its scene's main file's, `<name>.level.xrf.json`.
  pub const SIDECAR_SUFFIX: &'static str = Sidecar::SUFFIX;

  /// The provenance of a scene of `level` decompiled now by `tool`, recording no source, file or finding yet.
  pub fn new(level: impl Into<String>, tool: impl Into<String>, tool_version: impl Into<String>) -> Self {
    Self {
      level: level.into(),
      tool: tool.into(),
      tool_version: tool_version.into(),
      decompiled_at: SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs()),
      sources: Vec::new(),
      files: Vec::new(),
      inferred: Vec::new(),
      refused: Vec::new(),
      root_spawn: None,
      geometry: None,
    }
  }

  /// The sidecar's text.
  ///
  /// # Errors
  ///
  /// Returns an error when the provenance does not serialise, which it always does.
  pub fn to_json(&self) -> XrfResult<String> {
    Sidecar::format_json(self)
  }

  /// Where the sidecar of the scene whose main file is `level` stands: `<level>.xrf.json`, beside it.
  pub fn sidecar_path(level: &Path) -> PathBuf {
    Sidecar::get_path(level)
  }
}
