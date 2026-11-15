use xrf_editable_level::{
  EditableFogVolumeObject, EditableGlowObject, EditableLightObject, EditableObjectTransform, EditableParticleObject,
  EditablePortalObject, EditableShapeElement, EditableShapeObject, EditableSoundEnvironmentObject,
  EditableSoundSourceObject, EditableSpawnPointKind, EditableSpawnPointObject, EditableToolObject, EditableWayObject,
};
use xrf_math::Vector3d;
use xrf_renderer::RenderHelperIcon;

use crate::scene::{SceneHelperLook, SceneRenderSpace};

/// The section an actor's spawn point names, where the game starts the player.
const ACTOR_SECTION: &str = "actor";

/// An object tool's typed object the editor draws a helper for, as the SDK's editor marks what has no geometry.
pub(crate) trait SceneHelpedObject: EditableToolObject {
  /// What its helper draws and where; none for an object drawing none.
  fn get_look(&self) -> Option<SceneHelperLook>;
}

/// A spawn point by what it places: an entity's volume where a shape is attached to it, else an icon by what its
/// section or type says without the game's configs (the actor's start, a multiplayer respawn point, any other entity),
/// and an environment modifier by the sphere it reaches, as the level viewer draws `level.env_mod`'s.
impl SceneHelpedObject for EditableSpawnPointObject {
  fn get_look(&self) -> Option<SceneHelperLook> {
    let transform: &EditableObjectTransform = &self.object.transform;

    match &self.kind {
      EditableSpawnPointKind::RespawnPoint { .. } => Some(SceneHelperLook::icon_at(
        SceneRenderSpace::place(transform),
        RenderHelperIcon::RespawnPoint,
      )),
      EditableSpawnPointKind::EnvironmentModifier { radius, .. } => {
        SceneHelperLook::of_sphere(&transform.position, *radius).or_else(|| {
          Some(SceneHelperLook::icon_at(
            SceneRenderSpace::place(transform),
            RenderHelperIcon::SpawnPoint,
          ))
        })
      }
      EditableSpawnPointKind::Entity { section, .. } => {
        // The build gives an entity its spawn point's place and turn and the attached shape's elements with its scale
        // applied, so the volume stands as the game's object holds it.
        let volume: Option<SceneHelperLook> = self.attached.as_ref().and_then(|attached| {
          let elements: Vec<EditableShapeElement> = attached.get_scaled_elements();

          SceneHelperLook::of_elements(
            &EditableObjectTransform {
              scale: Vector3d::ONE,
              ..*transform
            },
            &elements,
          )
        });

        volume.or_else(|| {
          Some(SceneHelperLook::icon_at(
            SceneRenderSpace::place(transform),
            if section == ACTOR_SECTION {
              RenderHelperIcon::ActorStart
            } else {
              RenderHelperIcon::SpawnPoint
            },
          ))
        })
      }
    }
  }
}

/// A way by its links, arrowed, and a mark at each of its points, as the level viewer draws a patrol path.
impl SceneHelpedObject for EditableWayObject {
  fn get_look(&self) -> Option<SceneHelperLook> {
    SceneHelperLook::of_path(&self.points, RenderHelperIcon::PatrolPoint)
  }
}

/// A shape by its spheres and boxes.
impl SceneHelpedObject for EditableShapeObject {
  fn get_look(&self) -> Option<SceneHelperLook> {
    SceneHelperLook::of_elements(&self.object.transform, &self.shape.elements)
  }
}

/// A sound environment by its shape, whose first box splits its inner environment from its outer.
impl SceneHelpedObject for EditableSoundEnvironmentObject {
  fn get_look(&self) -> Option<SceneHelperLook> {
    SceneHelperLook::of_elements(&self.object.transform, &self.shape.elements)
  }
}

/// A fog volume by the unit box its transform alone places, which is what the build exports and the renderer fills,
/// whatever its shape's own elements.
impl SceneHelpedObject for EditableFogVolumeObject {
  fn get_look(&self) -> Option<SceneHelperLook> {
    Some(SceneHelperLook::of_unit_box(&self.object.transform))
  }
}

/// A light by its icon, as the level viewer draws a static light.
impl SceneHelpedObject for EditableLightObject {
  fn get_look(&self) -> Option<SceneHelperLook> {
    Some(SceneHelperLook::icon_at(
      SceneRenderSpace::place(&self.object.transform),
      RenderHelperIcon::Light,
    ))
  }
}

/// A glow by the sphere of the radius it is drawn and culled at, the light's icon for one of no radius.
impl SceneHelpedObject for EditableGlowObject {
  fn get_look(&self) -> Option<SceneHelperLook> {
    SceneHelperLook::of_sphere(&self.object.transform.position, self.radius).or_else(|| {
      Some(SceneHelperLook::icon_at(
        SceneRenderSpace::place(&self.object.transform),
        RenderHelperIcon::Light,
      ))
    })
  }
}

/// A sound source by its icon where the sound stands, its own position, which its transform does not move. Its
/// distances are not drawn: the scene saves them as they were when last read from the sound's file, a decompiled
/// scene the defaults, and the file is not read here.
impl SceneHelpedObject for EditableSoundSourceObject {
  fn get_look(&self) -> Option<SceneHelperLook> {
    Some(SceneHelperLook::icon_at(
      SceneRenderSpace::place_at(&self.position),
      RenderHelperIcon::Sound,
    ))
  }
}

/// A particle effect by its icon, as the level viewer draws a static one.
impl SceneHelpedObject for EditableParticleObject {
  fn get_look(&self) -> Option<SceneHelperLook> {
    Some(SceneHelperLook::icon_at(
      SceneRenderSpace::place(&self.object.transform),
      RenderHelperIcon::Particles,
    ))
  }
}

/// A portal by its polygon's outline, its corners being in world space.
impl SceneHelpedObject for EditablePortalObject {
  fn get_look(&self) -> Option<SceneHelperLook> {
    SceneHelperLook::of_outline(&self.vertices)
  }
}
