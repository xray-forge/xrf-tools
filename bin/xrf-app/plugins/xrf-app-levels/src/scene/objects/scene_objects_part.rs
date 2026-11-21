use std::collections::HashMap;
use std::sync::Arc;

use xrf_app_core::session::{SessionPart, SessionSnapshot};
use xrf_editable_level::{
  EditableFogVolumeObject, EditableGlowObject, EditableGroupTool, EditableLightObject, EditableParticleObject,
  EditablePortalObject, EditableSceneObject, EditableSectorObject, EditableShapeObject, EditableSoundEnvironmentObject,
  EditableSoundSourceObject, EditableSpawnPointObject, EditableToolObjects, EditableWayObject,
};
use xrf_error::XrfResult;
use xrf_vfs::XraySearch;

use crate::scene::{
  SceneObjectDescription, SceneObjectId, SceneObjectProperties, SceneObjectToolDescription, SceneObjectsDescription,
  SceneProblem, SceneProblemKind, SceneToolKind, SceneToolObject, SelectedScene,
};

/// The change event name for stale scene objects.
#[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
pub(crate) const SCENE_OBJECTS_PART: &str = "sceneObjects";

/// The scene's object tools' typed objects, read on first use from the parts the open kept. It records where it looked
/// for library objects, so an `.object` added or removed moves this part and not the scene.
pub(crate) struct SceneObjectsPart;

impl SessionPart for SceneObjectsPart {
  type Value = Arc<SceneObjectsDescription>;

  const NAME: &'static str = SCENE_OBJECTS_PART;
}

impl SceneObjectsPart {
  /// The open scene's object tools' objects, read on first use and kept, library objects looked for through `objects`,
  /// a search of the scene's objects roots.
  pub(crate) fn get(current: &SessionSnapshot<SelectedScene>, objects: &XraySearch) -> Arc<SceneObjectsDescription> {
    current
      .parts
      .get_or_record::<Self>(objects, |objects| Arc::new(Self::read(current, objects)))
  }

  /// Every object tool's objects, in the editor's tool order. A tool whose objects, group members included, do not read
  /// becomes a problem, as the editor fails that tool and loads the rest.
  pub(crate) fn read(scene: &SelectedScene, objects: &XraySearch) -> SceneObjectsDescription {
    xrf_profiler::scope!("scene objects");

    let mut description: SceneObjectsDescription = SceneObjectsDescription {
      tools: Vec::new(),
      problems: Vec::new(),
    };

    for kind in SceneToolKind::ALL {
      let read: XrfResult<Option<SceneObjectToolDescription>> = match kind {
        SceneToolKind::SceneObject => Self::read_tool::<EditableSceneObject>(kind, scene, objects),
        SceneToolKind::Sector => Self::read_tool::<EditableSectorObject>(kind, scene, objects),
        SceneToolKind::Portal => Self::read_tool::<EditablePortalObject>(kind, scene, objects),
        SceneToolKind::Light => Self::read_tool::<EditableLightObject>(kind, scene, objects),
        SceneToolKind::Glow => Self::read_tool::<EditableGlowObject>(kind, scene, objects),
        SceneToolKind::Spawn => Self::read_tool::<EditableSpawnPointObject>(kind, scene, objects),
        SceneToolKind::Shape => Self::read_tool::<EditableShapeObject>(kind, scene, objects),
        SceneToolKind::Way => Self::read_tool::<EditableWayObject>(kind, scene, objects),
        SceneToolKind::Particle => Self::read_tool::<EditableParticleObject>(kind, scene, objects),
        SceneToolKind::SoundSource => Self::read_tool::<EditableSoundSourceObject>(kind, scene, objects),
        SceneToolKind::SoundEnvironment => Self::read_tool::<EditableSoundEnvironmentObject>(kind, scene, objects),
        SceneToolKind::FogVolume => Self::read_tool::<EditableFogVolumeObject>(kind, scene, objects),
        // Groups count the members the other tools read, so they are read last.
        SceneToolKind::Group => continue,
        // Binary tools keep no objects; their counts are a part of their own.
        SceneToolKind::AiMap | SceneToolKind::DetailObject | SceneToolKind::Wallmark => continue,
      };

      Self::keep(&mut description, kind, read);
    }

    let groups: XrfResult<Option<SceneObjectToolDescription>> = Self::read_groups(scene, &description.tools);

    Self::keep(&mut description, SceneToolKind::Group, groups);
    description.tools.sort_by_key(|tool| tool.tool);

    description
  }

  /// One tool's objects in editor load order, group members first and named with their group; none where the scene has
  /// neither a part of the tool `kind` nor a group member of its class.
  fn read_tool<O: SceneToolObject>(
    kind: SceneToolKind,
    scene: &SelectedScene,
    objects: &XraySearch,
  ) -> XrfResult<Option<SceneObjectToolDescription>> {
    let read: EditableToolObjects<O> = EditableToolObjects::read(&scene.scene, scene.guid)?;

    if read.members.is_empty() && scene.scene.get_tool(O::TOOL_NAME).is_none() {
      return Ok(None);
    }

    let mut described: Vec<SceneObjectDescription> = read
      .iter_indexed()
      .map(|(index, object)| SceneObjectDescription::of(SceneObjectId { tool: kind, index }, object, objects))
      .collect();

    for (description, member) in described.iter_mut().zip(read.members) {
      description.group = Some(member.group);
    }

    Ok(Some(SceneObjectToolDescription {
      tool: kind,
      objects: described,
    }))
  }

  /// The group tool's groups, each counting the objects of `tools` naming it; none where the scene has no group tool.
  fn read_groups(
    scene: &SelectedScene,
    tools: &[SceneObjectToolDescription],
  ) -> XrfResult<Option<SceneObjectToolDescription>> {
    let Some(tool) = scene.scene.get_tool(EditableGroupTool::TOOL_NAME) else {
      return Ok(None);
    };
    let mut members: HashMap<&str, u32> = HashMap::new();

    for group in tools
      .iter()
      .flat_map(|tool| &tool.objects)
      .filter_map(|object| object.group.as_deref())
    {
      *members.entry(group).or_default() += 1;
    }

    let groups: Vec<SceneObjectDescription> = (0u32..)
      .zip(EditableGroupTool::read_groups(tool)?)
      .map(|(index, group)| {
        let properties: SceneObjectProperties = SceneObjectProperties::Group {
          members: members.get(group.name.as_str()).copied().unwrap_or_default(),
        };
        let id: SceneObjectId = SceneObjectId {
          tool: SceneToolKind::Group,
          index,
        };

        SceneObjectDescription::new(id, &group, properties)
      })
      .collect();

    Ok(Some(SceneObjectToolDescription {
      tool: SceneToolKind::Group,
      objects: groups,
    }))
  }

  /// Adds the tool `kind`'s `read` objects to `description`, or the reason they did not read.
  fn keep(
    description: &mut SceneObjectsDescription,
    kind: SceneToolKind,
    read: XrfResult<Option<SceneObjectToolDescription>>,
  ) {
    match read {
      Ok(Some(tool)) => description.tools.push(tool),
      Ok(None) => {}
      Err(error) => description.problems.push(SceneProblem {
        kind: SceneProblemKind::Unreadable,
        tool: kind.get_name().to_owned(),
        file: None,
        message: format!("Objects do not read: {error}"),
      }),
    }
  }
}
