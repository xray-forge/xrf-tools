use std::sync::Arc;

use xrf_app_core::session::{SessionPart, SessionSnapshot};
use xrf_chunk::XRayByteOrder;
use xrf_editable_level::{EditableAiMapTool, EditableDetailObjectTool, EditableSceneTool, EditableWallmarkTool};
use xrf_error::XrfResult;

use crate::scene::{
  SceneBinaryToolDescription, SceneBinaryToolsDescription, SceneProblem, SceneProblemKind, SceneToolKind, SelectedScene,
};

/// What a change event calls a scene's binary tools' counts when they go stale.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub(crate) const SCENE_BINARY_TOOLS_PART: &str = "sceneBinaryTools";

/// The scene's binary tools counted, read the first time anything asks from the parts its open kept: it reads no file of
/// its own, so it goes with the scene. Pooled, as parts are by default: a large AI map takes a while to read.
pub(crate) struct SceneBinaryToolsPart;

impl SessionPart for SceneBinaryToolsPart {
  type Value = Arc<SceneBinaryToolsDescription>;

  const NAME: &'static str = SCENE_BINARY_TOOLS_PART;
}

impl SceneBinaryToolsPart {
  /// The open scene's binary tools counted, read the first time anything asks and kept.
  pub(crate) fn get(current: &SessionSnapshot<SelectedScene>) -> Arc<SceneBinaryToolsDescription> {
    current.parts.get_or_init::<Self>(|| Arc::new(Self::read(current)))
  }

  /// Every binary tool the scene holds a part of, counted, in the order the tools are listed; a tool whose stream does
  /// not read is left out with a problem.
  pub(crate) fn read(scene: &SelectedScene) -> SceneBinaryToolsDescription {
    profiling::scope!("scene binary tools");

    let mut description: SceneBinaryToolsDescription = SceneBinaryToolsDescription {
      tools: Vec::new(),
      problems: Vec::new(),
    };

    for kind in SceneToolKind::ALL {
      let Some(tool) = scene.scene.get_tool(kind.get_name()) else {
        continue;
      };
      let read: XrfResult<SceneBinaryToolDescription> = match kind {
        SceneToolKind::AiMap => Self::count_ai_map(tool),
        SceneToolKind::DetailObject => Self::count_detail_objects(tool),
        SceneToolKind::Wallmark => Self::count_wallmarks(tool),
        _ => continue,
      };

      match read {
        Ok(counted) => description.tools.push(counted),
        Err(error) => description.problems.push(SceneProblem {
          kind: SceneProblemKind::Unreadable,
          tool: kind.get_name().to_owned(),
          file: None,
          message: format!("Tool's stream does not read: {error}"),
        }),
      }
    }

    description
  }

  fn count_ai_map(tool: &EditableSceneTool) -> XrfResult<SceneBinaryToolDescription> {
    let map: EditableAiMapTool = EditableAiMapTool::read_from_scene_tool::<XRayByteOrder>(tool)?;

    Ok(SceneBinaryToolDescription::AiMap {
      nodes: map.nodes.len() as u32,
    })
  }

  fn count_detail_objects(tool: &EditableSceneTool) -> XrfResult<SceneBinaryToolDescription> {
    let details: EditableDetailObjectTool = EditableDetailObjectTool::read_from_scene_tool::<XRayByteOrder>(tool)?;

    Ok(SceneBinaryToolDescription::DetailObject {
      objects: details.objects.len() as u32,
      slots: details.slots.len() as u32,
    })
  }

  fn count_wallmarks(tool: &EditableSceneTool) -> XrfResult<SceneBinaryToolDescription> {
    let wallmarks: EditableWallmarkTool = EditableWallmarkTool::read_from_scene_tool::<XRayByteOrder>(tool)?;

    Ok(SceneBinaryToolDescription::Wallmark {
      slots: wallmarks.slots.len() as u32,
      marks: wallmarks.slots.iter().map(|slot| slot.marks.len() as u32).sum(),
    })
  }
}
