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

/// An object tool's typed object the editor draws a helper for, marking what has no geometry as the SDK's editor does.
pub(crate) trait SceneHelpedObject: EditableToolObject {
  /// What its helper draws and where; none for an object drawing none.
  fn get_look(&self) -> Option<SceneHelperLook>;

  /// What its helper draws beside the model it is drawn as, in place of its icon; none by default, the model being all
  /// it shows.
  fn get_modelled_look(&self) -> Option<SceneHelperLook> {
    None
  }
}

/// A spawn point by what it places: an entity by its attached shape's volume, else an icon chosen by section or type
/// without the game's configs (actor start, multiplayer respawn point, other entity); an environment modifier by the
/// sphere it reaches, as the level viewer draws `level.env_mod`'s. An entity drawn as its model keeps its volume and
/// loses its icon.
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
      EditableSpawnPointKind::Entity { section, .. } => self.get_modelled_look().or_else(|| {
        Some(SceneHelperLook::icon_at(
          SceneRenderSpace::place(transform),
          if section == ACTOR_SECTION {
            RenderHelperIcon::ActorStart
          } else {
            RenderHelperIcon::SpawnPoint
          },
        ))
      }),
    }
  }

  /// An entity's attached shape volume, placed as the build gives it to the game object: the spawn point's position
  /// and rotation, and the shape's elements with its scale applied.
  fn get_modelled_look(&self) -> Option<SceneHelperLook> {
    self.attached.as_ref().and_then(|attached| {
      let elements: Vec<EditableShapeElement> = attached.get_scaled_elements();

      SceneHelperLook::of_elements(
        &EditableObjectTransform {
          scale: Vector3d::ONE,
          ..self.object.transform
        },
        &elements,
      )
    })
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

/// A fog volume by the unit box its transform places, which the build exports and the renderer fills whatever its
/// shape's elements.
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

/// A glow by the sphere of its draw and cull radius, or the light icon for one with no radius.
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

/// A sound source by its icon at its own position, which its transform does not move. Its distances are not drawn: the
/// scene saves them as last read from the sound file (a decompiled scene the defaults), which is not read here.
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
