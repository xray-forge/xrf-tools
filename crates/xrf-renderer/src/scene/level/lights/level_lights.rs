use std::collections::HashMap;
use std::sync::Arc;

use glam::{Vec2, Vec3, Vec4};
use xrf_math::{EPS_L, Vector3d};
use xrf_renderer_core::{GraphDeclaration, ProxyHandle, ProxyStore};
use xrf_visual::{LightAnimatorDescription, LightDescription, LightVolumetric, LightsDescription};

use crate::camera::camera_frustum::CameraFrustum;
use crate::feature::lighting::lights::lights_feature::LightsFeature;
use crate::feature::scene::drawable::casters::drawable_kind_ref::DrawableKindRef;
use crate::frame::frame_encoder::FrameEncoder;
use crate::frame::periodic::periodic_scheduler::PeriodicScheduler;
use crate::host::level::zone_fast_mode::ZONE_FAST_DISTANCE;
use crate::host::render_asset_source::RenderAssetSource;
use crate::host::world::render_motion_pose::RenderMotionPose;
use crate::lighting::lamp::light_basis::LightBasis;
use crate::scene::level::lights::asks::light_asks::LightAsks;
use crate::scene::level::lights::asks::light_in_view::{LightFades, LightInView};
use crate::scene::level::lights::asks::lights_frame::LightsFrame;
use crate::scene::level::lights::atlas::level_light_shadows::LevelLightShadows;
use crate::scene::level::lights::atlas::light_shadow_ask::LightShadowAsk;
use crate::scene::level::lights::atlas::light_shadow_atlas::LightShadowAtlas;
use crate::scene::level::shadow::shadow_frame::ShadowFrame;
use crate::scene::level::shadow::shadow_sway::ShadowSway;
use crate::scene::level::view::record::view_record::ViewRecord;
use crate::scene::static_scene::static_scene::StaticScene;
use crate::scene::texture::texture_cache::TextureCache;
use crate::scene::texture::texture_role::TextureRole;

mod level_lights_records;

/// Metres from the eye to a shadowed light's reach within which, out of view, its faces are kept ready for a turn, and
/// how many such lights are.
const WARM_REACH: f32 = 3.0;
const WARM_MOST: usize = 8;

/// What a light's falloff reaches zero at, a share of its range: `L_R` (`r3_rendertarget_accum_point.cpp`).
const FALLOFF_RANGE: f32 = 0.95;

/// The noise breaking the shafts (`s_noise`), and the projector Monolith draws every light's shafts through
/// (`s_accum_volume`, `r4_rendertarget.cpp`), which a point's faces and a spot without its own take in OpenXRay too.
const VOLUME_NOISE: &str = "fx\\fx_noise";
const VOLUME_PROJECTOR: &str = "lights\\lights_spot01";

/// `VOLUMETRIC_SLICES`: the slices the engines' buffer holds, of which a light's shafts draw some.
const VOLUME_SLICES: u32 = 100;

/// `fDensity`: what one slice of a light's shafts adds of its light, as `light_volumes.wgsl` holds it too.
const VOLUME_DENSITY: f32 = 1.0 / 40.0;

/// The engine's own shaft quality, intensity and distance for a light whose owner sets none (`light::light`).
const VOLUME_DEFAULTS: LightVolumetric = LightVolumetric {
  is_asked: false,
  quality: 1.0,
  intensity: 1.0,
  distance: 1.0,
};

/// OpenXRay's fewest slices a light's shafts draw (`_max(10, iNumSlices)`).
const VOLUME_FEWEST_SLICES: u32 = 10;

/// Monolith's `ps_ssfx_volumetric` as it ships, which forces every light's shafts on: their intensity and quality, its
/// slices a quality (`24 * fQuality`), and the most a quality draws (`24 * 5`), which its depth is stretched by.
const MONOLITH_INTENSITY: f32 = 1.0;
const MONOLITH_QUALITY: f32 = 3.0;
const MONOLITH_SLICES_PER_QUALITY: f32 = 24.0;
const MONOLITH_MOST_SLICES: f32 = 120.0;

/// Metres from the eye Monolith's shafts start fading at, and the share of their intensity a metre past it takes.
const MONOLITH_FADE_START: f32 = 20.0;
const MONOLITH_FADE_RATE: f32 = 0.01;

/// What Monolith scales an omni part's shafts by (`IntensityMod`).
const MONOLITH_PART_INTENSITY: f32 = 0.2;

/// A level's local lights: read once on a loader thread, moved by their motions, and each frame the nearest in a view
/// written out into its `LightsFeature` in its view space, animated as the engine animates them, for the lights pass to
/// bin and accumulate; and the shadow faces they cast, cached in an atlas.
pub(crate) struct LevelLights {
  /// The level's lights, each reached by the handle it was added under.
  lights: ProxyStore<LightDescription>,
  /// Those a motion carries, which move each frame.
  moving: Vec<ProxyHandle<LightDescription>>,
  /// The animations replacing lights' colours, by the index a light names.
  animators: Vec<LightAnimatorDescription>,
  /// Each projector's texture slot, by its index among the description's projectors.
  projectors: Vec<u32>,
  /// The slots of the shafts' noise and of the projector Monolith draws them through.
  volume_textures: (u32, u32),
  /// Where the level's campfires burn, by their idle lights' `x` and `z`.
  fires: Vec<Vec2>,
  shadows: LevelLightShadows,
}

impl LevelLights {
  pub(crate) fn new(device: &wgpu::Device, view_layout: &wgpu::BindGroupLayout) -> Self {
    Self {
      lights: ProxyStore::new(),
      moving: Vec::new(),
      animators: Vec::new(),
      projectors: Vec::new(),
      volume_textures: (TextureCache::MISSING_SLOT, TextureCache::MISSING_SLOT),
      fires: Vec::new(),
      shadows: LevelLightShadows::new(device, view_layout),
    }
  }

  /// The texture slots the projectors sample.
  pub(crate) fn get_projectors(&self) -> &[u32] {
    &self.projectors
  }

  /// Where the level's campfires burn, lit or out, by `x` and `z`.
  pub(crate) fn get_fires(&self) -> &[Vec2] {
    &self.fires
  }

  /// Takes a level's lights in place of any it held, asking for every projector they name.
  pub(crate) fn add_lights(
    &mut self,
    lights: LightsDescription,
    textures: &mut TextureCache,
    source: &Arc<dyn RenderAssetSource>,
  ) {
    self.projectors = lights
      .projectors
      .iter()
      .map(|reference| textures.request(reference, TextureRole::Projector, source))
      .collect();
    self.volume_textures = (
      textures.request(VOLUME_NOISE, TextureRole::Projector, source),
      textures.request(VOLUME_PROJECTOR, TextureRole::Projector, source),
    );
    log::info!("Native viewport lights {} local lights", lights.lights.len());
    self.animators = lights.animators;
    // The store hands its handles out again from the first, so no face of a light before may stay keyed by one.
    self.lights = ProxyStore::new();
    self.moving.clear();
    self.fires.clear();
    self.shadows.forget_lights();

    for light in lights.lights {
      let is_moving: bool = light.motion.is_some();

      if light.campfire.is_some() {
        self.fires.push(Vec2::new(light.position.x, light.position.z));
      }

      let handle: ProxyHandle<LightDescription> = self.lights.add(light);

      if is_moving {
        self.moving.push(handle);
      }
    }
  }

  /// Picks the lights a view sees this frame by its camera and settings, nearest first, as many as its records hold, and
  /// the shadowed ones out of view the eye stands near, kept ready, each faded as far as its campfire's share in
  /// `campfire_shares` lets it show: what the view asks of them, which the scene settles with every other view's
  /// (`settle`) before the view writes its lights (`write`).
  pub(crate) fn pick(&self, frame: LightsFrame, campfire_shares: &HashMap<u16, f32>) -> LightAsks {
    let (mut in_view, mut ready): (Vec<LightInView>, Vec<LightInView>) = if frame.settings.is_enabled {
      let mut in_view: Vec<LightInView> = self.collect_in_view(&frame, campfire_shares);

      in_view.sort_by(|a, b| a.distance.total_cmp(&b.distance));
      in_view.into_iter().partition(|it| it.is_seen)
    } else {
      (Vec::new(), Vec::new())
    };
    let excess: u32 = in_view.len().saturating_sub(LightsFeature::MAX_LIGHTS) as u32;

    in_view.truncate(LightsFeature::MAX_LIGHTS);
    ready.truncate(WARM_MOST);

    LightAsks {
      frame,
      forward: -frame.camera.view.inverse().z_axis.truncate(),
      in_view,
      ready,
      excess,
    }
  }

  /// Settles what every view of the scene asked of its lights this frame, the driving view's first, whose casters the
  /// faces are drawn with: every casting view's shadowed lights in view marked seen first, so a full atlas makes room
  /// from none of them; then each casting view's nearest lights its records hold ask for faces, then the nearest it
  /// keeps ready out of view, over the `statics` they cast from.
  pub(crate) fn settle(&mut self, views: &[&LightAsks], statics: &StaticScene) {
    for asks in views.iter().filter(|asks| asks.frame.is_shadowing()) {
      for it in &asks.in_view {
        if self.lights.get(it.handle).is_some_and(|light| light.is_shadowed) {
          self.shadows.see(it.handle);
        }
      }
    }

    for (index, asks) in views.iter().enumerate() {
      let frame: &LightsFrame = &asks.frame;

      if !frame.is_shadowing() {
        continue;
      }

      self.shadows.cast();

      let sway: ShadowSway<'_> = ShadowSway {
        amplitude: frame.sway_amplitude,
        reach: statics.get_sway_reach(),
        time: frame.time,
        places: statics.list_swaying(),
      };

      for (it, is_in_view) in asks
        .in_view
        .iter()
        .map(|it| (it, true))
        .chain(asks.ready.iter().map(|it| (it, false)))
      {
        let Some(light) = self.lights.get(it.handle).filter(|light| light.is_shadowed) else {
          continue;
        };

        self.shadows.ask(&LightShadowAsk {
          handle: it.handle,
          light,
          basis: &it.basis,
          eye: frame.camera.position,
          forward: asks.forward,
          is_in_view,
          is_driving: index == 0,
          statics,
          sway: &sway,
          casters: frame.casters.as_slice(),
        });
      }
    }
  }

  /// The lights a view sees this frame, by its camera and settings: in its frustum, or, out of it, a shadowed one the
  /// eye stands near, its faces kept ready for a turn; each with its fades by the cull's levels of detail and its
  /// campfire's share; none a campfire or a torrid zone has put out.
  fn collect_in_view(&self, frame: &LightsFrame, campfire_shares: &HashMap<u16, f32>) -> Vec<LightInView> {
    let LightsFrame {
      camera, settings, lod, ..
    } = *frame;
    let is_shadowing: bool = frame.is_shadowing();
    let frustum: CameraFrustum = camera.get_frustum();
    let eye: Vec3 = camera.position;

    self
      .lights
      .iter()
      .filter(|(_, light)| settings.is_level_lights || !light.is_level)
      .filter_map(|(handle, light)| {
        // A campfire's idle light: out while it is, faded over a turn (`UpdateWorkload`).
        let share: f32 = light
          .campfire
          .map_or(1.0, |id| campfire_shares.get(&id).copied().unwrap_or(1.0));

        if share <= 0.0 {
          return None;
        }

        // A torrid zone's idle light: out while the camera stands far from its zone (`o_switch_2_slow`).
        if let Some(sphere) = &light.zone_sphere {
          let at: [f32; 3] = [light.position.x, light.position.y, light.position.z];

          if sphere.get_distance(at, eye.to_array()) > ZONE_FAST_DISTANCE {
            return None;
          }
        }

        let basis: LightBasis = LightBasis::of(light);
        let bound: Vec4 = basis.get_bound(light);
        let is_visible: bool = frustum.reaches(bound);

        let distance: f32 = (eye.distance(bound.truncate()) - bound.w).max(0.0);

        // Out of view, only a shadowed light the eye stands near is kept, its faces ready for a turn.
        if !is_visible && !(is_shadowing && light.is_shadowed && distance <= WARM_REACH) {
          return None;
        }

        let fades: LightFades = if is_shadowing && light.is_shadowed {
          Self::to_shadow_fades(light, &basis, eye, lod)
        } else {
          LightFades::WHOLE
        };

        // `light::get_LOD`: a shadowed light is drawn at all only past `EPS_L`.
        (fades.shown > EPS_L).then_some(LightInView {
          distance,
          is_seen: is_visible,
          handle,
          basis,
          bound,
          fades,
          share,
        })
      })
      .collect()
  }

  /// Starts a scene's frame: stands each light its motion carries.
  pub(crate) fn begin_frame(&mut self, motions: &HashMap<String, RenderMotionPose>) {
    self.move_lights(motions);
  }

  /// Stands each light a motion carries `height` over where the motion has its zone this frame (`UpdateIdleLight`);
  /// one whose motion is still read stays where its zone spawned.
  fn move_lights(&mut self, motions: &HashMap<String, RenderMotionPose>) {
    for handle in &self.moving {
      let Some(motion) = self.lights.get(*handle).and_then(|light| light.motion.clone()) else {
        continue;
      };
      let Some(pose) = motions.get(&motion.name) else {
        continue;
      };
      let at: Vec3 = pose.transform.w_axis.truncate();

      // Lights stand in renderer space, which mirrors the engine's z.
      if let Some(light) = self.lights.get_mut(*handle) {
        light.position = Vector3d::new(at.x, at.y + motion.height, -at.z);
      }
    }
  }

  /// Ends what every view of the scene asked of the shadows this frame: offers `scheduler` the faces wanting a draw.
  pub(crate) fn offer_shadows(&mut self, scheduler: &mut PeriodicScheduler) {
    self.shadows.finish(scheduler);
  }

  /// Readies the faces `scheduler` granted, which `add_shadow_passes` then draws before the lights read them; then
  /// opens the next frame's asking.
  pub(crate) fn commit_shadows(
    &mut self,
    device: &wgpu::Device,
    encoder: &mut FrameEncoder,
    frame: &ShadowFrame<'_>,
    scheduler: &PeriodicScheduler,
  ) {
    self.shadows.prepare(device, encoder, frame, scheduler);
    self.shadows.begin();
  }

  /// Forgets the faces this frame drew, where its frame was not submitted.
  pub(crate) fn abandon_shadows(&mut self) {
    self.shadows.abandon_frame();
  }

  /// Declares the culls and draws of the shadow faces this frame's `prepare_shadows` readied, what the drawable `kinds`
  /// cast over those casting from them.
  pub(crate) fn add_shadow_passes<'a>(
    &'a self,
    declaration: GraphDeclaration<'_, 'a>,
    record: ViewRecord<'a>,
    kinds: &[DrawableKindRef<'a>],
  ) {
    self.shadows.add_passes(declaration, record, kinds);
  }

  pub(crate) fn get_shadow_atlas(&self) -> LightShadowAtlas<'_> {
    self.shadows.get_atlas()
  }
}

/// A shadowed light's shafts as the view's engine gives them out this frame.
#[derive(Clone, Copy)]
pub(crate) struct LightShafts {
  is_spot: bool,
  /// The slices drawn, a point's widened by √2 for its faces.
  slices: u32,
  /// The share of the buffer the volume's depth is stretched over.
  quality: f32,
  /// The colour's scale.
  intensity: f32,
  /// How far towards the light its sphere is pulled.
  distance: f32,
  /// The sphere the slices fill, in the world.
  sphere: Vec4,
}

impl LightShafts {
  /// What a metre of the shafts adds of the light where all of it reaches: the slices standing within the sphere over
  /// its depth, spaced by the buffer's steps over the depth the quality stretches (`vs_light_volume`), each adding
  /// `VOLUME_DENSITY` at the colour's scale; the distance shrinks the sphere and scales the colour alike, so it falls
  /// out.
  pub(crate) fn get_density(&self) -> f32 {
    (VOLUME_SLICES - 1) as f32 * self.quality * VOLUME_DENSITY * self.intensity / (2.0 * self.sphere.w).max(EPS_L)
  }
}
