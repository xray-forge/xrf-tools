use crate::contract::render_view_options::RenderViewOptions;
use xrf_renderer_core::PassMarker;

use crate::contract::render_water_mode::RenderWaterMode;
use crate::contract::render_water_settings::RenderWaterSettings;
use crate::frame::view_targets::ViewTargets;
use crate::frame::water_reflection::WaterReflection;
use crate::frame::water_scene::WaterScene;
use crate::lighting::render_wind::RenderWind;
use crate::pass::static_draw_groups::StaticDrawGroups;
use crate::pass::view_binding::ViewBinding;
use crate::pass::water_groups::WaterGroups;
use crate::pass::water_pass::WaterPass;
use crate::pass::water_sources::WaterSources;
use crate::pass::water_uniform::WaterUniform;
use crate::scene::level::water_flow::WaterFlow;

/// What a level view's water reads of its frame beside its own state: the targets and their epoch, the lighting, both
/// skies with the sampler and version they were bound at, the weather's `water_intensity`, wind and rain density, and the
/// clock.
pub struct WaterFrame<'a> {
  pub targets: Option<(&'a ViewTargets, u64)>,
  pub lighting: &'a wgpu::Buffer,
  pub skies: ([&'a wgpu::TextureView; 2], &'a wgpu::Sampler, u64),
  pub intensity: f32,
  pub wind: RenderWind,
  pub rain: f32,
  pub time: f32,
}

/// A level view's water: its settings and uniform, the scene before it that the enhanced water refracts, the enhanced
/// water's reflection, and what binds them, each made only while the frame draws what needs it.
pub struct LevelWater {
  uniform: wgpu::Buffer,
  settings: RenderWaterSettings,
  /// Whether this frame draws the water at all: it is on, and the view is not wireframe.
  is_drawn: bool,
  scene: Option<WaterScene>,
  reflection: Option<WaterReflection>,
  /// With the sky's version, the targets' epoch, and whether they bind the scene and the reflection.
  groups: Option<((u64, u64, bool, bool), WaterGroups)>,
  /// How far the enhanced water's maps have scrolled.
  flow: WaterFlow,
}

impl LevelWater {
  pub fn new(device: &wgpu::Device) -> Self {
    Self {
      uniform: device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("water"),
        size: size_of::<WaterUniform>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      }),
      settings: RenderWaterSettings::default(),
      is_drawn: false,
      scene: None,
      reflection: None,
      groups: None,
      flow: WaterFlow::default(),
    }
  }

  /// Takes this frame's settings: makes or drops the scene copy and the reflection, binds what draws, and writes the
  /// uniform.
  pub fn prepare(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    pass: &WaterPass,
    options: &RenderViewOptions,
    frame: WaterFrame<'_>,
  ) {
    let settings: RenderWaterSettings = options.water;

    self.settings = settings;
    self.is_drawn = settings.is_enabled && !options.is_wireframe;
    self.flow.advance(frame.time, &settings, frame.wind);

    let is_refracting: bool = self.is_drawn && settings.mode == RenderWaterMode::Enhanced;
    let is_reflecting: bool = is_refracting && settings.reflectivity > 0.0;

    match frame.targets {
      Some((targets, epoch)) => {
        if !is_refracting {
          self.scene = None;
        } else if self.scene.as_ref().is_none_or(|scene| scene.epoch != epoch) {
          self.scene = Some(WaterScene::new(device, targets, epoch));
        }

        if !is_reflecting {
          self.reflection = None;
        } else if self
          .reflection
          .as_ref()
          .is_none_or(|reflection| reflection.epoch != epoch)
        {
          self.reflection = Some(WaterReflection::new(device, targets, epoch));
        }

        let (skies, sky_sampler, sky_version) = frame.skies;
        let key: (u64, u64, bool, bool) = (sky_version, epoch, is_refracting, is_reflecting);

        if self.groups.as_ref().is_none_or(|(bound, _)| *bound != key) {
          let groups: WaterGroups = pass.create_bind_groups(
            device,
            &WaterSources {
              targets,
              lighting: frame.lighting,
              water: &self.uniform,
              skies,
              sky_sampler,
              scene: self.scene.as_ref().map(|scene| &scene.view),
              reflection: self.reflection.as_ref(),
            },
          );

          self.groups = Some((key, groups));
        }
      }
      None => {
        self.scene = None;
        self.reflection = None;
        self.groups = None;
      }
    }

    queue.write_buffer(
      &self.uniform,
      0,
      bytemuck::bytes_of(&WaterUniform::new(
        &settings,
        (frame.intensity, frame.wind, frame.rain),
        (frame.time, self.flow),
        self
          .reflection
          .as_ref()
          .map_or((false, false), |reflection| (true, reflection.is_valid)),
      )),
    );
  }

  /// Draws the water over the scene drawn so far, its stages timed, copying that scene first for the enhanced water,
  /// and makes the reflection just drawn the one the next frame keeps.
  #[allow(clippy::too_many_arguments)]
  pub fn record(
    &mut self,
    encoder: &mut wgpu::CommandEncoder,
    pass: &WaterPass,
    targets: &ViewTargets,
    (view, draw_groups, texture_group): (&ViewBinding, &StaticDrawGroups, &wgpu::BindGroup),
    args: &[&wgpu::Buffer],
    timer: &mut PassMarker<'_>,
  ) {
    let Some((_, groups)) = self.groups.as_ref().filter(|_| self.is_drawn) else {
      return;
    };

    if let Some(scene) = &self.scene {
      scene.copy(encoder, targets);
      timer.mark(encoder, "water copy");
    }

    pass.draw(
      encoder,
      targets,
      view,
      draw_groups,
      texture_group,
      (groups, self.settings.mode, self.reflection.as_ref()),
      args,
      &mut |encoder, name| timer.mark(encoder, name),
    );

    if let Some(reflection) = &mut self.reflection {
      reflection.swap();
    }

    timer.mark(encoder, "water");
  }
}
