//! A Level Editor scene as a document beside the compiled level: its `.level` and its tools' part files read through a
//! search of the directory holding the `.level`, so a commit to any of them reads it again (`document`); what the page
//! is shown of it (`description`); its object tools' typed objects (`objects`) and its binary tools' counts (`binary`),
//! each a part of its session read on demand with a query of its own; what the editor would skip or refuse of a part,
//! kept as a problem on its tool rather than refusing the open (`problem`); the scene objects it draws, where they
//! stand and which it cannot draw (`draw`); and the scene as the native renderer draws it, each scene object a model
//! compiled from its `.object`, with the source each session is drawn from (`render`).

mod binary;
mod description;
mod document;
mod draw;
mod objects;
mod problem;
mod render;

pub(crate) use binary::{SceneBinaryToolDescription, SceneBinaryToolsDescription, SceneBinaryToolsPart};
pub(crate) use description::{
  SceneBuildParamsDescription, SceneCameraDescription, ScenePartFileDescription, SceneToolDescription, SceneToolFormat,
  SelectedSceneDescription,
};
pub use document::SelectedScenes;
pub(crate) use document::{OpenedScene, SceneGameData, SceneOpenRequest, SceneSource, SceneToolKind, SelectedScene};
pub(crate) use draw::{
  SceneDrawDescription, SceneDrawPart, SceneDrawProblem, SceneDrawProblemKind, SceneDrawnObject, SceneDrawnObjects,
};
pub(crate) use objects::{
  SceneLightType, SceneObjectDescription, SceneObjectId, SceneObjectProperties, SceneObjectToolDescription,
  SceneObjectTransform, SceneObjectsDescription, SceneObjectsPart, SceneSectorItem, SceneShape, SceneShapeElement,
  SceneShapeUsage, SceneSpawnField, SceneSpawnKind, SceneToolObject, SceneWayLink, SceneWayPoint, SceneWayType,
};
pub(crate) use problem::{SceneProblem, SceneProblemKind};
pub(crate) use render::SceneRenderSource;
pub use render::ShownScenes;
