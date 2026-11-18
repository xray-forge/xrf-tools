//! A Level Editor scene as a document beside the compiled level: its `.level` and its tools' part files read through a
//! search of the directory holding the `.level`, so a commit to any of them reads it again (`document`); what the page
//! is shown of it (`description`); its object tools' typed objects (`objects`) and its binary tools' counts (`binary`),
//! each a part of its session read on demand with a query of its own; what the editor would skip or refuse of a part,
//! kept as a problem on its tool rather than refusing the open (`problem`); the scene objects it draws, where they
//! stand and which it cannot draw (`draw`); the spawn points it draws as their entities' models and the visuals they
//! are read as (`spawn`); the helpers drawn for what has no geometry, each named by its object (`helpers`); and the
//! scene as the native renderer draws it, each scene object a model compiled from its `.object`, with the source each
//! session is drawn from, the one id every drawn thing of an object carries, the space it stands in and where one
//! object stands for a camera to frame it (`render`).

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
