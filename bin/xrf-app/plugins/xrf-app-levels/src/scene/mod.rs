//! A Level Editor scene as a document beside the compiled level: its `.level` and its tools' part files read through a
//! search of the directory holding the `.level`, so a commit to any of them reads it again (`document`); what the page
//! is shown of it (`description`); its object tools' typed objects (`objects`) and its binary tools' counts (`binary`),
//! each a part of its session read on demand with a query of its own; and what the editor would skip or refuse of a
//! part, kept as a problem on its tool rather than refusing the open (`problem`).

mod binary;
mod description;
mod document;
mod objects;
mod problem;

pub(crate) use binary::{SceneBinaryToolDescription, SceneBinaryToolsDescription, SceneBinaryToolsPart};
pub(crate) use description::{
  SceneBuildParamsDescription, SceneCameraDescription, ScenePartFileDescription, SceneToolDescription, SceneToolFormat,
  SelectedSceneDescription,
};
pub use document::SelectedScenes;
pub(crate) use document::{OpenedScene, SceneOpenRequest, SceneSource, SceneToolKind, SelectedScene};
pub(crate) use objects::{
  SceneLightType, SceneObjectDescription, SceneObjectId, SceneObjectProperties, SceneObjectToolDescription,
  SceneObjectTransform, SceneObjectsDescription, SceneObjectsPart, SceneSectorItem, SceneShape, SceneShapeElement,
  SceneShapeUsage, SceneSpawnField, SceneSpawnKind, SceneToolObject, SceneWayLink, SceneWayPoint, SceneWayType,
};
pub(crate) use problem::{SceneProblem, SceneProblemKind};
