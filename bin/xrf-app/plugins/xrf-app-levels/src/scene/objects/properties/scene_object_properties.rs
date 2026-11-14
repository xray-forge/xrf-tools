use serde::Serialize;
use xrf_editable_level::{
  EditableFogVolumeObject, EditableGlowObject, EditableLightObject, EditableParticleObject, EditablePortalObject,
  EditableSectorObject, EditableShapeObject, EditableSoundEnvironmentObject, EditableSoundSourceObject,
  EditableSpawnPointObject, EditableWayObject,
};
use xrf_math::{Vector2d, Vector3d};

use crate::scene::{SceneLightType, SceneSectorItem, SceneShape, SceneSpawnKind, SceneWayPoint, SceneWayType};

/// What one scene object's kind keeps beyond its name and transform, as an inspector shows it: one variant per object
/// tool.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub(crate) enum SceneObjectProperties {
  /// A library object placed, `scene_object.part`.
  SceneObject {
    /// The `.object` it places, a path below the objects roots without its extension.
    reference: String,
    flags: u32,
    /// Whether its `.object` was found in the objects roots the scene was opened with.
    is_found: bool,
  },
  /// The meshes the renderer culls together, `sector.part`.
  Sector {
    /// The colour the editor draws it in: red, green, blue and alpha bytes.
    color: [u8; 4],
    /// Whether it is the sector everything no other sector holds falls into.
    is_default: bool,
    /// The level map index it switches to; none for one switching to none.
    map_index: Option<u8>,
    items: Vec<SceneSectorItem>,
  },
  /// A polygon one sector is seen through from another, `portal.part`.
  Portal {
    sector_front: String,
    sector_back: String,
    /// Its corners in world space, in order.
    vertices: Vec<Vector3d>,
  },
  /// A light, `light.part`.
  Light {
    light_type: SceneLightType,
    /// Red, green, blue and alpha, lit with times the brightness.
    color: [f32; 4],
    brightness: f32,
    range: f32,
    attenuation: [f32; 3],
    /// A spot's cone angle, in radians.
    cone: f32,
    virtual_size: f32,
    is_used_in_d3d: bool,
    /// Whether it bakes, lights at run time, and the editor's other bits, the fuzzy one aside.
    flags: u32,
    /// The light control it answers to, by its index in the light tool's controls.
    control: u32,
    /// The light animation it plays, empty for none.
    animation: String,
    falloff_texture: String,
    /// The points a fuzzy point light spreads over, in its own space; none for another light.
    fuzzy_points: Vec<Vector3d>,
  },
  /// A light flare, `glow.part`.
  Glow {
    radius: f32,
    shader: String,
    texture: String,
    flags: u16,
  },
  /// A spawn point, `spawn.part`, with the shape attached where its entity needs a volume.
  Spawn {
    spawn: SceneSpawnKind,
    attached: Option<SceneShape>,
  },
  /// A shape the shape tool places, `shape.part`.
  Shape { shape: SceneShape },
  /// A patrol path, `way.part`.
  Way {
    way_type: SceneWayType,
    points: Vec<SceneWayPoint>,
  },
  /// A particle effect planted, `ps.part`.
  Particle {
    /// The effect it plays, a definition of `particles.xr`.
    reference: String,
    /// The game modes it plays in, one bit each.
    game_types: u16,
  },
  /// A sound planted, `sound_src.part`.
  SoundSource {
    /// The sound it plays, a path below the game's sounds without its extension.
    sound: String,
    flags: u32,
    /// Where the sound stands, which its transform does not move.
    position: Vector3d,
    volume: f32,
    frequency: f32,
    min_distance: f32,
    max_distance: f32,
    max_ai_distance: f32,
    /// The pause between playings, in seconds.
    random_pause: Vector2d,
    /// The window of the game's day it plays in, in seconds from midnight.
    active_time: Vector2d,
    /// How long one playing lasts, in seconds.
    play_time: Vector2d,
  },
  /// A volume of reverb, `sound_env.part`: its first box splits the inner environment from the outer.
  SoundEnvironment {
    shape: SceneShape,
    inner: String,
    outer: String,
  },
  /// A body of volumetric fog, `fog_volume.part`.
  FogVolume {
    shape: SceneShape,
    /// The emitter's profile; none for an occlusion volume.
    profile: Option<String>,
    /// Which volumes export together; none for one never grouped.
    group: Option<u32>,
  },
  /// A group, `group.part`, whose members are described under their own tools.
  Group {
    /// How many members of the tools described name it as their group.
    members: u32,
  },
}

impl From<&EditableSectorObject> for SceneObjectProperties {
  fn from(sector: &EditableSectorObject) -> Self {
    Self::Sector {
      color: sector.color,
      is_default: sector.is_default,
      map_index: (sector.map_index != EditableSectorObject::NO_MAP).then_some(sector.map_index),
      items: sector.items.iter().map(SceneSectorItem::from).collect(),
    }
  }
}

impl From<&EditablePortalObject> for SceneObjectProperties {
  fn from(portal: &EditablePortalObject) -> Self {
    Self::Portal {
      sector_front: portal.sector_front.clone(),
      sector_back: portal.sector_back.clone(),
      vertices: portal.vertices.clone(),
    }
  }
}

impl From<&EditableLightObject> for SceneObjectProperties {
  fn from(light: &EditableLightObject) -> Self {
    Self::Light {
      light_type: SceneLightType::of(light.kind),
      color: light.color,
      brightness: light.brightness,
      range: light.range,
      attenuation: light.attenuation,
      cone: light.cone,
      virtual_size: light.virtual_size,
      is_used_in_d3d: light.use_in_d3d,
      flags: light.flags,
      control: light.control,
      animation: light.animation.clone(),
      falloff_texture: light.falloff_texture.clone(),
      fuzzy_points: light
        .fuzzy
        .as_ref()
        .map(|fuzzy| fuzzy.points.clone())
        .unwrap_or_default(),
    }
  }
}

impl From<&EditableGlowObject> for SceneObjectProperties {
  fn from(glow: &EditableGlowObject) -> Self {
    Self::Glow {
      radius: glow.radius,
      shader: glow.shader.clone(),
      texture: glow.texture.clone(),
      flags: glow.flags,
    }
  }
}

impl From<&EditableSpawnPointObject> for SceneObjectProperties {
  fn from(spawn: &EditableSpawnPointObject) -> Self {
    Self::Spawn {
      spawn: SceneSpawnKind::from(&spawn.kind),
      attached: spawn
        .attached
        .as_ref()
        .map(|attached| SceneShape::from(&attached.shape)),
    }
  }
}

impl From<&EditableShapeObject> for SceneObjectProperties {
  fn from(shape: &EditableShapeObject) -> Self {
    Self::Shape {
      shape: SceneShape::from(&shape.shape),
    }
  }
}

impl From<&EditableWayObject> for SceneObjectProperties {
  fn from(way: &EditableWayObject) -> Self {
    Self::Way {
      way_type: SceneWayType::of(way.kind),
      points: way.points.iter().map(SceneWayPoint::from).collect(),
    }
  }
}

impl From<&EditableParticleObject> for SceneObjectProperties {
  fn from(particle: &EditableParticleObject) -> Self {
    Self::Particle {
      reference: particle.reference.clone(),
      game_types: particle.game_types,
    }
  }
}

impl From<&EditableSoundSourceObject> for SceneObjectProperties {
  fn from(sound: &EditableSoundSourceObject) -> Self {
    Self::SoundSource {
      sound: sound.sound.clone(),
      flags: sound.flags,
      position: sound.position,
      volume: sound.volume,
      frequency: sound.frequency,
      min_distance: sound.min_distance,
      max_distance: sound.max_distance,
      max_ai_distance: sound.max_ai_distance,
      random_pause: sound.random_pause,
      active_time: sound.active_time,
      play_time: sound.play_time,
    }
  }
}

impl From<&EditableSoundEnvironmentObject> for SceneObjectProperties {
  fn from(environment: &EditableSoundEnvironmentObject) -> Self {
    Self::SoundEnvironment {
      shape: SceneShape::from(&environment.shape),
      inner: environment.inner.clone(),
      outer: environment.outer.clone(),
    }
  }
}

impl From<&EditableFogVolumeObject> for SceneObjectProperties {
  fn from(volume: &EditableFogVolumeObject) -> Self {
    Self::FogVolume {
      shape: SceneShape::from(&volume.shape),
      profile: volume.get_profile().map(str::to_owned),
      group: (volume.group != EditableFogVolumeObject::UNGROUPED).then_some(volume.group),
    }
  }
}
