use std::collections::HashMap;
use std::sync::Arc;
use xrf_level::LevelDynamicLight;
use xrf_light_anim::{LightAnimFile, LightAnimItem, LightAnimKey};
use xrf_ltx::{Ltx, Section};
use xrf_math::Vector3d;

use xrf_spawn::{AlifeObject, AlifeObjectHangingLamp, AlifeObjectInherited};

use crate::data::lights::light_animator_description::LightAnimatorDescription;
use crate::data::lights::light_animator_key::LightAnimatorKey;
use crate::data::lights::light_description::LightDescription;
use crate::data::lights::light_kind::LightKind;
use crate::data::lights::lights_description::LightsDescription;
use crate::data::visual::skeleton::bind_transform::BindTransform;
use crate::data::visual::skeleton::visual_rest_pose::VisualRestPose;
use crate::data::visual::skeleton::visual_transform::VisualTransform;
use crate::pack::visual_conversion::convert_vector;

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
}

impl<'a> LightsPacker<'a> {
  /// The binder of a signal rocket (`bind_signal_light.script`), which turns its lamp off on its first update: lit only
  /// while a scripted launch flies it.
  const SIGNAL_LIGHT_BINDING: &'static str = "bind_signal_light.init";

  /// The section vanilla binds it to, which is what a lamp is known by without configs to read its binding from.
  const SIGNAL_LIGHT_SECTION: &'static str = "lights_signal_light";

  /// What a spot with no projector of its own projects (`r2_rendertarget.cpp`).
  const DEFAULT_PROJECTOR: &'static str = "lights\\lights_spot01";

  /// `idle_light_range_delta`'s default (xray-16's `CCustomZone::Load`), and the only value xray-monolith knows.
  const ZONE_RANGE_JITTER: f32 = 0.25;

  /// The widest cone a spot takes here, so its projection stays finite; the engines pass any cone through.
  const MAX_CONE: f32 = 120.0 * std::f32::consts::PI / 180.0;

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
    }
  }

  /// Reads each spawned object's section from the game's configs: a zone's idle light, and a lamp's own `shadow` and
  /// `ambient_shadow`.
  pub fn with_sections(mut self, sections: &'a Ltx) -> Self {
    self.sections = Some(sections);
    self
  }

  /// Adds the lights a level's spawned objects carry, each by its class: a hanging lamp's, and a zone's idle light.
  ///
  /// # Arguments
  ///
  /// * `objects` - The objects spawned on the level.
  /// * `pose_visual` - The rest pose of a visual by the name an object gives it, `None` for one that cannot be read;
  ///   asked once per lamp, so a caller reading from disk keeps what it read.
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
    for light in lights.iter().filter(|light| !light.is_sun()) {
      self.lights.push(LightDescription {
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

  /// The lights added, with the animators and projectors they name.
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
    let section: Option<&Section> = self.find_section(&object.section);

    if !lamp.is_spawned_on_r2() || lamp.health <= 0.0 || self.is_signal_rocket(object, section) {
      return;
    }

    let visual: Option<Arc<VisualRestPose>> = object.inherited.get_visual().and_then(pose_visual);
    let pose: Option<&VisualRestPose> = visual.as_deref();
    let object_transform: VisualTransform = VisualTransform::of_spawn(&object.position, &object.direction);
    let main: VisualTransform = Self::place_on_bone(&object_transform, pose, &lamp.light_bone);
    let ambient: VisualTransform = if lamp.light_ambient_bone.eq_ignore_ascii_case(&lamp.light_bone) {
      main.clone()
    } else {
      Self::place_on_bone(&object_transform, pose, &lamp.light_ambient_bone)
    };
    let color: [f32; 3] = Self::unpack_color(lamp.main_color, lamp.main_brightness);
    let animator: Option<u32> = self.find_animator(&lamp.color_animator);
    let is_spot: bool = lamp.is_spot();
    let projector: Option<u32> = is_spot.then(|| {
      self.find_projector(if lamp.light_texture.is_empty() {
        Self::DEFAULT_PROJECTOR
      } else {
        &lamp.light_texture
      })
    });

    self.lights.push(LightDescription {
      kind: if is_spot { LightKind::Spot } else { LightKind::Point },
      position: main.c.clone(),
      direction: Self::to_direction(&main),
      right: main.i.clone(),
      color,
      range: lamp.main_range,
      range_jitter: 0.0,
      cone: lamp.spot_cone_angle.min(Self::MAX_CONE),
      near: lamp.virtual_size,
      projector,
      animator,
      animator_scale: lamp.main_brightness / 255.0,
      is_shadowed: section
        .and_then(|it| it.get_bool("shadow"))
        .unwrap_or_else(|| lamp.casts_shadow()),
      is_level: false,
    });

    if lamp.has_point_ambient() {
      self.lights.push(LightDescription {
        kind: LightKind::Point,
        position: ambient.c.clone(),
        direction: Self::to_direction(&ambient),
        right: ambient.i.clone(),
        color: color.map(|channel| channel * lamp.ambient_power),
        range: lamp.ambient_radius,
        range_jitter: 0.0,
        cone: 0.0,
        near: 0.0,
        projector: None,
        animator,
        animator_scale: lamp.main_brightness / 255.0 * lamp.ambient_power,
        is_shadowed: section.and_then(|it| it.get_bool("ambient_shadow")).unwrap_or(false),
        is_level: false,
      });
    }
  }

  /// `CCustomZone::Load` and `StartIdleLight`: a zone whose section lights it a point light `idle_light_height` over
  /// it, its colour its animation's alone, its range straying each frame. One whose animation the library lacks, which
  /// the engine would refuse, lights nothing.
  pub(crate) fn add_zone(&mut self, object: &AlifeObject) {
    let Some(section) = self.find_section(&object.section) else {
      return;
    };

    if section.get_bool("idle_light") != Some(true) {
      return;
    }

    let Some(range) = section.get_f32("idle_light_range") else {
      return;
    };
    let range_jitter: f32 = section
      .get_f32("idle_light_range_delta")
      .unwrap_or(Self::ZONE_RANGE_JITTER);
    let height: f32 = section.get_f32("idle_light_height").unwrap_or(0.0);
    let is_shadowed: bool = section.get_bool("idle_light_shadow").unwrap_or(true);
    let Some(animator) = section
      .get("idle_light_anim")
      .map(str::trim)
      .and_then(|name| self.find_animator(name))
    else {
      return;
    };
    let position: Vector3d = Vector3d::new(object.position.x, object.position.y + height, object.position.z);

    self.lights.push(LightDescription {
      kind: LightKind::Point,
      position: convert_vector(&position),
      direction: Vector3d::new(0.0, 0.0, 1.0),
      right: Vector3d::new(1.0, 0.0, 0.0),
      color: [0.0, 0.0, 0.0],
      range,
      range_jitter,
      cone: 0.0,
      near: 0.0,
      projector: None,
      animator: Some(animator),
      animator_scale: 1.0 / 255.0,
      is_shadowed,
      is_level: false,
    });
  }

  /// Whether a lamp is a signal rocket, by the binder its section names, or by vanilla's section without configs.
  fn is_signal_rocket(&self, object: &AlifeObject, section: Option<&Section>) -> bool {
    match self.sections {
      Some(_) => section
        .and_then(|it| it.get("script_binding"))
        .is_some_and(|binding| binding.trim().eq_ignore_ascii_case(Self::SIGNAL_LIGHT_BINDING)),
      None => object.section == Self::SIGNAL_LIGHT_SECTION,
    }
  }

  /// A spawned object's section, from configs outliving the packer rather than borrowed through it, so a caller can
  /// hold it while the packer adds animators.
  fn find_section(&self, name: &str) -> Option<&'a Section> {
    self.sections?.section(name)
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
        self.animators.push(Self::describe_animator(item, file.is_bgr()));

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

  /// The engine's `xf.k`, where a light points, in the renderer's space: mirroring a transform negates the `x` and `y` of
  /// its third axis, so negating that axis gives the converted `k` back.
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
}
