use std::collections::HashMap;
use std::sync::Arc;
use xrf_level::LevelDynamicLight;
use xrf_light_anim::{LightAnimFile, LightAnimItem, LightAnimKey};
use xrf_ltx::Ltx;
use xrf_math::Vector3d;

use xrf_spawn::{AlifeObject, AlifeObjectHangingLamp, AlifeObjectInherited};

use crate::data::lights::light_animator_description::LightAnimatorDescription;
use crate::data::lights::light_animator_key::LightAnimatorKey;
use crate::data::lights::light_description::LightDescription;
use crate::data::lights::light_kind::LightKind;
use crate::data::lights::lights_description::LightsDescription;
use crate::data::visual::skeleton::visual_rest_pose::VisualRestPose;
use crate::data::visual::skeleton::visual_transform::VisualTransform;
use crate::pack::visual::visual_transform::{BindTransform, to_spawn_transform};
use crate::pack::visual_conversion::convert_vector;

/// `CSE_ALifeObjectHangingLamp::flCastShadow`.
const FLAG_CAST_SHADOW: u16 = 1 << 1;

/// `CSE_ALifeObjectHangingLamp::flR2`: a lamp the engine spawns on R2 and later.
const FLAG_R2: u16 = 1 << 3;

/// `CSE_ALifeObjectHangingLamp::flTypeSpot`.
const FLAG_SPOT: u16 = 1 << 4;

/// `CSE_ALifeObjectHangingLamp::flPointAmbient`: a second, unshadowed point light at the ambient bone.
const FLAG_POINT_AMBIENT: u16 = 1 << 5;

/// A signal rocket: a hanging lamp its script (`bind_signal_light`) turns off on its first update, lit only while a
/// scripted launch flies it.
const SIGNAL_LIGHT_SECTION: &str = "lights_signal_light";

/// What a spot with no projector of its own projects (`r2_rendertarget.cpp`).
const DEFAULT_PROJECTOR: &str = "lights\\lights_spot01";

/// `idle_light_range_delta`'s default (`CCustomZone::Load`).
const ZONE_RANGE_JITTER: f32 = 0.25;

/// The widest cone `light::spatial_move` accepts.
const MAX_CONE: f32 = 120.0 * std::f32::consts::PI / 180.0;

/// Collects a level's lights as the engine would light with them: its hanging lamps, placed on their bones, and the
/// lights of the level file itself.
pub struct LightsPacker<'a> {
  animations: Option<&'a LightAnimFile>,
  /// The game's `system.ltx`, resolved, which a spawned object's section is read from.
  sections: Option<&'a Ltx>,
  lights: Vec<LightDescription>,
  animators: Vec<LightAnimatorDescription>,
  /// Each animation name looked up, and where it landed, `None` for one the library does not hold.
  animator_indices: HashMap<String, Option<u32>>,
  projectors: Vec<String>,
  projector_indices: HashMap<String, u32>,
  /// Each visual read, `None` for one that could not be.
  visuals: HashMap<String, Option<Arc<VisualRestPose>>>,
}

impl<'a> LightsPacker<'a> {
  /// A packer resolving colour animations against `lanims.xr`, or animating nothing without one.
  pub fn new(animations: Option<&'a LightAnimFile>) -> Self {
    Self {
      animations,
      sections: None,
      lights: Vec::new(),
      animators: Vec::new(),
      animator_indices: HashMap::new(),
      projectors: Vec::new(),
      projector_indices: HashMap::new(),
      visuals: HashMap::new(),
    }
  }

  /// Reads each spawned object's section from the game's configs: a zone's idle light, and a lamp's own `shadow`.
  pub fn with_sections(mut self, sections: &'a Ltx) -> Self {
    self.sections = Some(sections);
    self
  }

  /// Adds the lights a level's spawned objects carry, each by its class: a hanging lamp's, and a zone's idle light.
  ///
  /// # Arguments
  ///
  /// * `objects` - The objects spawned on the level.
  /// * `pose_visual` - The rest pose of a visual by the name an object gives it, `None` for one that cannot be read.
  pub fn add_objects(
    &mut self,
    objects: &[AlifeObject],
    pose_visual: &mut dyn FnMut(&str) -> Option<Arc<VisualRestPose>>,
  ) {
    for object in objects {
      match &object.inherited {
        AlifeObjectInherited::CseAlifeObjectHangingLamp(lamp) => self.add_lamp(object, lamp, pose_visual),
        AlifeObjectInherited::CseAlifeAnomalousZone(_)
        | AlifeObjectInherited::CseAlifeZoneVisual(_)
        | AlifeObjectInherited::CseAlifeTorridZone(_) => self.add_zone(object),
        _ => {}
      }
    }
  }

  /// Adds the level file's own point lights, which the engine draws only with `r2_allow_r1_lights`: its position,
  /// range and colour, always shadowed. The directional one is the sun, lit elsewhere.
  pub fn add_level_lights(&mut self, lights: &[LevelDynamicLight]) {
    for (index, light) in lights.iter().enumerate().filter(|(_, light)| !light.is_sun()) {
      self.lights.push(LightDescription {
        name: format!("level light {index}"),
        kind: LightKind::Point,
        position: convert_vector(&light.position),
        direction: Vector3d::new(0.0, 0.0, 1.0),
        right: Vector3d::new(1.0, 0.0, 0.0),
        color: [light.diffuse.r, light.diffuse.g, light.diffuse.b],
        range: light.range,
        range_jitter: 0.0,
        cone: 0.0,
        near: 0.0,
        projector: None,
        animator: None,
        animator_scale: 0.0,
        is_shadowed: true,
        is_level: true,
      });
    }
  }

  pub fn pack(self) -> LightsDescription {
    LightsDescription {
      lights: self.lights,
      animators: self.animators,
      projectors: self.projectors,
    }
  }

  /// `CHangingLamp::net_Spawn`: the main light on its bone, and the ambient one on its own where the lamp asks for
  /// it. A lamp the engine would not spawn on R2, one already broken, or a signal rocket waiting for launch lights
  /// nothing.
  pub(crate) fn add_lamp(
    &mut self,
    object: &AlifeObject,
    lamp: &AlifeObjectHangingLamp,
    pose_visual: &mut dyn FnMut(&str) -> Option<Arc<VisualRestPose>>,
  ) {
    if lamp.light_flags & FLAG_R2 == 0 || lamp.health <= 0.0 || object.section == SIGNAL_LIGHT_SECTION {
      return;
    }

    let visual_name: &str = &lamp.base.visual_name;

    if !self.visuals.contains_key(visual_name) {
      let visual: Option<Arc<VisualRestPose>> = if visual_name.is_empty() {
        None
      } else {
        pose_visual(visual_name)
      };

      self.visuals.insert(visual_name.to_owned(), visual);
    }

    let pose: Option<&VisualRestPose> = self.visuals.get(visual_name).and_then(Option::as_deref);
    let object_transform: VisualTransform = to_spawn_transform(&object.position, &object.direction);
    let main: VisualTransform = place_on_bone(&object_transform, pose, &lamp.light_bone);
    let ambient: VisualTransform = if lamp.light_ambient_bone.eq_ignore_ascii_case(&lamp.light_bone) {
      main.clone()
    } else {
      place_on_bone(&object_transform, pose, &lamp.light_ambient_bone)
    };
    let color: [f32; 3] = unpack_color(lamp.main_color, lamp.main_brightness);
    let animator: Option<u32> = self.find_animator(&lamp.color_animator);
    let is_spot: bool = lamp.light_flags & FLAG_SPOT != 0;
    let projector: Option<u32> = is_spot.then(|| {
      self.find_projector(if lamp.light_texture.is_empty() {
        DEFAULT_PROJECTOR
      } else {
        &lamp.light_texture
      })
    });

    self.lights.push(LightDescription {
      name: object.name.clone(),
      kind: if is_spot { LightKind::Spot } else { LightKind::Point },
      position: main.c.clone(),
      direction: to_direction(&main),
      right: main.i.clone(),
      color,
      range: lamp.main_range,
      range_jitter: 0.0,
      cone: lamp.spot_cone_angle.min(MAX_CONE),
      near: lamp.virtual_size,
      projector,
      animator,
      animator_scale: lamp.main_brightness / 255.0,
      // Anomaly's engine takes the section's own `shadow` over the flag where it names one.
      is_shadowed: self
        .read_bool(&object.section, "shadow")
        .unwrap_or(lamp.light_flags & FLAG_CAST_SHADOW != 0),
      is_level: false,
    });

    if lamp.light_flags & FLAG_POINT_AMBIENT != 0 {
      self.lights.push(LightDescription {
        name: format!("{} ambient", object.name),
        kind: LightKind::Point,
        position: ambient.c.clone(),
        direction: to_direction(&ambient),
        right: ambient.i.clone(),
        color: color.map(|channel| channel * lamp.ambient_power),
        range: lamp.ambient_radius,
        range_jitter: 0.0,
        cone: 0.0,
        near: 0.0,
        projector: None,
        animator,
        animator_scale: lamp.main_brightness / 255.0 * lamp.ambient_power,
        is_shadowed: false,
        is_level: false,
      });
    }
  }

  /// `CCustomZone::Load` and `StartIdleLight`: a zone whose section lights it a point light `idle_light_height` over
  /// it, its colour its animation's alone, its range straying each frame. One whose animation the library lacks, which
  /// the engine would refuse, lights nothing.
  pub(crate) fn add_zone(&mut self, object: &AlifeObject) {
    if self.read_bool(&object.section, "idle_light") != Some(true) {
      return;
    }

    let Some(range) = self.read_f32(&object.section, "idle_light_range") else {
      return;
    };
    let name: Option<String> = self.read_string(&object.section, "idle_light_anim");
    let Some(animator) = name.as_deref().and_then(|name| self.find_animator(name)) else {
      return;
    };
    let height: f32 = self.read_f32(&object.section, "idle_light_height").unwrap_or(0.0);
    let position: Vector3d = Vector3d::new(object.position.x, object.position.y + height, object.position.z);

    self.lights.push(LightDescription {
      name: object.name.clone(),
      kind: LightKind::Point,
      position: convert_vector(&position),
      direction: Vector3d::new(0.0, 0.0, 1.0),
      right: Vector3d::new(1.0, 0.0, 0.0),
      color: [0.0, 0.0, 0.0],
      range,
      range_jitter: self
        .read_f32(&object.section, "idle_light_range_delta")
        .unwrap_or(ZONE_RANGE_JITTER),
      cone: 0.0,
      near: 0.0,
      projector: None,
      animator: Some(animator),
      animator_scale: 1.0 / 255.0,
      is_shadowed: self.read_bool(&object.section, "idle_light_shadow").unwrap_or(true),
      is_level: false,
    });
  }

  fn read_string(&self, section: &str, key: &str) -> Option<String> {
    Some(self.sections?.section(section)?.get(key)?.trim().to_owned())
  }

  fn read_f32(&self, section: &str, key: &str) -> Option<f32> {
    self.read_string(section, key)?.parse().ok()
  }

  /// `CInifile::r_bool`: `on`, `yes`, `true` and `1` are true, anything else false.
  fn read_bool(&self, section: &str, key: &str) -> Option<bool> {
    self.read_string(section, key).map(|value| {
      ["on", "yes", "true", "1"]
        .iter()
        .any(|it| value.eq_ignore_ascii_case(it))
    })
  }

  /// `LALib.FindItem`: the animation by its exact name, added the first time a lamp names it.
  fn find_animator(&mut self, name: &str) -> Option<u32> {
    if name.is_empty() {
      return None;
    }

    if let Some(index) = self.animator_indices.get(name) {
      return *index;
    }

    let animations: Option<&LightAnimFile> = self.animations;
    let found: Option<u32> = animations
      .and_then(|file| {
        file
          .items
          .iter()
          .find(|item| item.name == name)
          .map(|item| (file, item))
      })
      .map(|(file, item)| {
        self.animators.push(describe_animator(item, file.is_bgr()));

        (self.animators.len() - 1) as u32
      });

    self.animator_indices.insert(name.to_owned(), found);

    found
  }

  fn find_projector(&mut self, reference: &str) -> u32 {
    if let Some(index) = self.projector_indices.get(reference) {
      return *index;
    }

    let index: u32 = self.projectors.len() as u32;

    self.projectors.push(reference.to_owned());
    self.projector_indices.insert(reference.to_owned(), index);

    index
  }
}

/// `XFORM() * LL_GetTransform(bone)` in the renderer's space: the bone's rest transform placed with the object, or the
/// object's own where the visual has no such bone.
fn place_on_bone(object: &VisualTransform, pose: Option<&VisualRestPose>, bone: &str) -> VisualTransform {
  pose
    .filter(|_| !bone.is_empty())
    .and_then(|pose| pose.find(bone))
    .map_or_else(
      || object.clone(),
      |it| {
        BindTransform::from_renderer_space(it)
          .then(&BindTransform::from_renderer_space(object))
          .to_visual()
      },
    )
}

/// The engine's `xf.k`, where a light points, in the renderer's space: a mirrored transform's third axis is the
/// mirrored axis turned about, since the mirror flips the third axis on both sides of it.
fn to_direction(transform: &VisualTransform) -> Vector3d {
  Vector3d::new(-transform.k.x, -transform.k.y, -transform.k.z)
}

/// `Fcolor(color)` times the lamp's brightness: an `ARGB` colour, each channel over 255, its alpha dropped.
fn unpack_color(color: u32, brightness: f32) -> [f32; 3] {
  [16, 8, 0].map(|shift| ((color >> shift) & 0xFF) as f32 / 255.0 * brightness)
}

/// The keys as the engine holds them once loaded, each as the `RGB` a lamp takes: a version 0 library stores `BGR`,
/// which the load swaps, and `CHangingLamp` swaps `CalculateBGR`'s result back.
fn describe_animator(item: &LightAnimItem, is_bgr: bool) -> LightAnimatorDescription {
  let mut keys: Vec<&LightAnimKey> = item.keys.iter().collect();

  keys.sort_by_key(|key| key.frame);

  LightAnimatorDescription {
    name: item.name.clone(),
    fps: item.fps,
    frame_count: item.frame_count,
    keys: keys
      .into_iter()
      .map(|key| {
        let shifts: [u32; 3] = if is_bgr { [0, 8, 16] } else { [16, 8, 0] };

        LightAnimatorKey {
          frame: key.frame,
          color: shifts.map(|shift| ((key.color >> shift) & 0xFF) as f32),
        }
      })
      .collect(),
  }
}
