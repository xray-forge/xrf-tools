use std::collections::HashMap;

use xrf_level::LevelDynamicLight;
use xrf_light_anim::{LightAnimFile, LightAnimItem, LightAnimKey};
use xrf_math::Vector3d;
use xrf_ogf::OgfFile;
use xrf_spawn::{AlifeObject, AlifeObjectHangingLamp, AlifeObjectInherited};

use crate::data::lights::light_animator_description::LightAnimatorDescription;
use crate::data::lights::light_animator_key::LightAnimatorKey;
use crate::data::lights::light_description::LightDescription;
use crate::data::lights::light_kind::LightKind;
use crate::data::lights::lights_description::LightsDescription;
use crate::pack::visual::visual_skeleton::resolve_bone_transform;
use crate::pack::visual::visual_transform::BindTransform;
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

/// The widest cone `light::spatial_move` accepts.
const MAX_CONE: f32 = 120.0 * std::f32::consts::PI / 180.0;

/// Collects a level's lights as the engine would light with them: its hanging lamps, placed on their bones, and the
/// lights of the level file itself.
pub struct LightsPacker<'a> {
  animations: Option<&'a LightAnimFile>,
  lights: Vec<LightDescription>,
  animators: Vec<LightAnimatorDescription>,
  /// Each animation name looked up, and where it landed, `None` for one the library does not hold.
  animator_indices: HashMap<String, Option<u32>>,
  projectors: Vec<String>,
  projector_indices: HashMap<String, u32>,
  /// Each visual read, `None` for one that could not be.
  visuals: HashMap<String, Option<OgfFile>>,
}

impl<'a> LightsPacker<'a> {
  /// A packer resolving colour animations against `lanims.xr`, or animating nothing without one.
  pub fn new(animations: Option<&'a LightAnimFile>) -> Self {
    Self {
      animations,
      lights: Vec::new(),
      animators: Vec::new(),
      animator_indices: HashMap::new(),
      projectors: Vec::new(),
      projector_indices: HashMap::new(),
      visuals: HashMap::new(),
    }
  }

  /// Adds the lights a level's spawned objects carry, each by its class: a hanging lamp's today.
  ///
  /// # Arguments
  ///
  /// * `objects` - The objects spawned on the level.
  /// * `read_visual` - Reads a visual by the name an object gives it, `None` for one that cannot be read.
  pub fn add_objects(&mut self, objects: &[AlifeObject], read_visual: &mut dyn FnMut(&str) -> Option<OgfFile>) {
    for object in objects {
      if let AlifeObjectInherited::CseAlifeObjectHangingLamp(lamp) = &object.inherited {
        self.add_lamp(object, lamp, read_visual);
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
    read_visual: &mut dyn FnMut(&str) -> Option<OgfFile>,
  ) {
    if lamp.light_flags & FLAG_R2 == 0 || lamp.health <= 0.0 || object.section == SIGNAL_LIGHT_SECTION {
      return;
    }

    let visual_name: &str = &lamp.base.visual_name;

    if !self.visuals.contains_key(visual_name) {
      let visual: Option<OgfFile> = if visual_name.is_empty() {
        None
      } else {
        read_visual(visual_name)
      };

      self.visuals.insert(visual_name.to_owned(), visual);
    }

    let visual: Option<&OgfFile> = self.visuals.get(visual_name).and_then(Option::as_ref);
    let object_transform: BindTransform = BindTransform::from_angle(&object.direction, &object.position);
    let main: BindTransform = place_on_bone(&object_transform, visual, &lamp.light_bone);
    let ambient: BindTransform = if lamp.light_ambient_bone.eq_ignore_ascii_case(&lamp.light_bone) {
      main.clone()
    } else {
      place_on_bone(&object_transform, visual, &lamp.light_ambient_bone)
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
      position: convert_vector(&main.c),
      direction: convert_vector(&main.k),
      right: convert_vector(&main.i),
      color,
      range: lamp.main_range,
      cone: lamp.spot_cone_angle.min(MAX_CONE),
      near: lamp.virtual_size,
      projector,
      animator,
      animator_scale: lamp.main_brightness / 255.0,
      is_shadowed: lamp.light_flags & FLAG_CAST_SHADOW != 0,
      is_level: false,
    });

    if lamp.light_flags & FLAG_POINT_AMBIENT != 0 {
      self.lights.push(LightDescription {
        name: format!("{} ambient", object.name),
        kind: LightKind::Point,
        position: convert_vector(&ambient.c),
        direction: convert_vector(&ambient.k),
        right: convert_vector(&ambient.i),
        color: color.map(|channel| channel * lamp.ambient_power),
        range: lamp.ambient_radius,
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

/// `XFORM() * LL_GetTransform(bone)`: the bone's rest transform placed with the object, or the object's own where the
/// visual has no such bone.
fn place_on_bone(object: &BindTransform, visual: Option<&OgfFile>, bone: &str) -> BindTransform {
  visual
    .filter(|_| !bone.is_empty())
    .and_then(|visual| {
      let bones = visual.bones.as_ref()?;

      resolve_bone_transform(
        &bones.bones,
        visual.ik_data.as_ref().map(|it| it.bones.as_slice()),
        bone,
      )
    })
    .map_or_else(|| object.clone(), |it| it.then(object))
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
