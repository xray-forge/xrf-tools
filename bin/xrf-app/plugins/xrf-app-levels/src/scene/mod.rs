//! Level Editor scenes as documents beside the compiled level. `document` reads a `.level` and its tools' part files
//! through a search of its directory, so a commit to any of them reads it again; `description` is what the page shows;
//! `objects` and `binary` hold the tools' typed objects and counts, each a session part with its own query; `problem`
//! keeps what the editor would skip or refuse of a part on its tool instead of failing the open; `draw` places the
//! scene objects; `spawn` draws spawn points as their entities' models; `helpers` marks what has no geometry; `render`
//! draws the scene in the native renderer, each scene object a model compiled from its `.object`, with the id, space
//! and camera framing every drawn object shares.

mod binary;
mod description;
mod document;
mod draw;
mod helpers;
mod objects;
mod problem;
mod render;
mod spawn;

pub(crate) use binary::{SceneBinaryToolDescription, SceneBinaryToolsDescription, SceneBinaryToolsPart};
pub(crate) use description::{
  SceneBuildParamsDescription, SceneCameraDescription, ScenePartFileDescription, SceneToolDescription, SceneToolFormat,
  SelectedSceneDescription,
};
pub use document::SelectedScenes;
pub(crate) use document::{OpenedScene, SceneGameData, SceneOpenRequest, SceneSource, SceneToolKind, SelectedScene};
pub(crate) use draw::{
  SceneDraw, SceneDrawDescription, SceneDrawPart, SceneDrawProblem, SceneDrawProblemKind, SceneDrawnObject,
  SceneDrawnObjects, SceneDrawnObjectsPart,
};
pub(crate) use helpers::{
  SCENE_HELPER_TOOLS, SceneHelpedObject, SceneHelperLook, SceneHelperToolCount, SceneHelperToolEntry, SceneHelpers,
  SceneHelpersDescription, SceneHelpersPart,
};
pub(crate) use objects::{
  SceneLightType, SceneObjectDescription, SceneObjectId, SceneObjectProperties, SceneObjectToolDescription,
  SceneObjectTransform, SceneObjectsDescription, SceneObjectsPart, SceneSectorItem, SceneShape, SceneShapeElement,
  SceneShapeUsage, SceneSpawnField, SceneSpawnKind, SceneToolObject, SceneWayLink, SceneWayPoint, SceneWayType,
};
pub(crate) use problem::{SceneProblem, SceneProblemKind};
pub use render::ShownScenes;
pub(crate) use render::{
  SceneObjectCompiler, SceneObjectModels, SceneObjectParts, SceneObjectSphere, SceneOccluderLook, SceneRenderId,
  SceneRenderSource, SceneRenderSpace, SceneSurfaceKind, SceneSurfaceKinds,
};
pub(crate) use spawn::{
  SceneSpawnModel, SceneSpawnModelDescription, SceneSpawnModels, SceneSpawnModelsPart, SceneSpawnVisualsPart,
};
