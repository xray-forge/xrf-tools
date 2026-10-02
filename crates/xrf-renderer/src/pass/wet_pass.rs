use xrf_error::XrfResult;

use crate::frame::view_targets::ViewTargets;
use crate::pass::fullscreen_pipeline::{buffer_binding, create_fullscreen_pipeline, texture_binding};
use crate::pass::layout_entries::{texture_entry, uniform_entry};
use crate::pass::view_binding::ViewBinding;
use crate::shader::shader_library::ShaderLibrary;

/// Rain on the G-buffer before any light, as `draw_rain` wets it (`r3_rendertarget_draw_rain.cpp`): where the rain
/// reaches near the camera its normals patched into the light target, borrowed before the sun clears it, then written
/// back, then the albedo darkened and the gloss raised by how wet it is.
pub struct WetPass {
  view_layout: wgpu::BindGroupLayout,
  /// The patch's, then the write back's and the wetting's.
  layouts: [wgpu::BindGroupLayout; 2],
  sampler: wgpu::Sampler,
  /// The patch, the normal written back, the albedo and gloss wetted.
  pipelines: [wgpu::RenderPipeline; 3],
  generation: u64,
}

impl WetPass {
  /// # Errors
  ///
  /// Returns an error when a shader does not compose or compile.
  pub fn new(device: &wgpu::Device, shaders: &ShaderLibrary, view_layout: &wgpu::BindGroupLayout) -> XrfResult<Self> {
    let fragment: wgpu::ShaderStages = wgpu::ShaderStages::FRAGMENT;
    let unfiltered: wgpu::TextureSampleType = wgpu::TextureSampleType::Float { filterable: false };
    let filtered: wgpu::TextureSampleType = wgpu::TextureSampleType::Float { filterable: true };
    let depth: wgpu::TextureSampleType = wgpu::TextureSampleType::Depth;
    let flat: wgpu::TextureViewDimension = wgpu::TextureViewDimension::D2;
    let patch: wgpu::BindGroupLayout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("wet patch"),
      entries: &[
        texture_entry(0, fragment, depth, flat),
        texture_entry(1, fragment, unfiltered, flat),
        texture_entry(2, fragment, unfiltered, flat),
        texture_entry(3, fragment, depth, flat),
        texture_entry(4, fragment, filtered, wgpu::TextureViewDimension::D2Array),
        texture_entry(5, fragment, filtered, flat),
        wgpu::BindGroupLayoutEntry {
          binding: 6,
          visibility: fragment,
          ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
          count: None,
        },
        uniform_entry(7, fragment),
      ],
    });
    let apply: wgpu::BindGroupLayout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("wet apply"),
      entries: &[
        texture_entry(0, fragment, depth, flat),
        texture_entry(1, fragment, unfiltered, flat),
        uniform_entry(2, fragment),
      ],
    });
    let layouts: [wgpu::BindGroupLayout; 2] = [patch, apply];

    Ok(Self {
      pipelines: Self::create_pipelines(device, shaders, view_layout, &layouts)?,
      sampler: device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("wet"),
        address_mode_u: wgpu::AddressMode::Repeat,
        address_mode_v: wgpu::AddressMode::Repeat,
        address_mode_w: wgpu::AddressMode::Repeat,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
      }),
      view_layout: view_layout.clone(),
      generation: shaders.get_generation(),
      layouts,
    })
  }

  pub fn refresh(&mut self, device: &wgpu::Device, shaders: &ShaderLibrary) {
    if shaders.get_generation() != self.generation {
      self.generation = shaders.get_generation();

      match Self::create_pipelines(device, shaders, &self.view_layout, &self.layouts) {
        Ok(pipelines) => self.pipelines = pipelines,
        Err(error) => log::error!("Wet surfaces rejected, wetting with the last ones: {error}"),
      }
    }
  }

  /// The patch's bind group, then the write back's and the wetting's.
  pub fn create_bind_groups(
    &self,
    device: &wgpu::Device,
    targets: &ViewTargets,
    cover: &wgpu::TextureView,
    (splash, flow): (&wgpu::TextureView, &wgpu::TextureView),
    uniform: &wgpu::Buffer,
  ) -> [wgpu::BindGroup; 2] {
    [
      device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("wet patch"),
        layout: &self.layouts[0],
        entries: &[
          texture_binding(0, &targets.depth),
          texture_binding(1, &targets.albedo),
          texture_binding(2, &targets.normal),
          texture_binding(3, cover),
          texture_binding(4, splash),
          texture_binding(5, flow),
          wgpu::BindGroupEntry {
            binding: 6,
            resource: wgpu::BindingResource::Sampler(&self.sampler),
          },
          buffer_binding(7, uniform),
        ],
      }),
      device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("wet apply"),
        layout: &self.layouts[1],
        entries: &[
          texture_binding(0, &targets.depth),
          texture_binding(1, &targets.light),
          buffer_binding(2, uniform),
        ],
      }),
    ]
  }

  pub fn draw(
    &self,
    encoder: &mut wgpu::CommandEncoder,
    targets: &ViewTargets,
    view: &ViewBinding,
    [patch, apply]: &[wgpu::BindGroup; 2],
  ) {
    for (index, (target, group)) in [
      (&targets.light, patch),
      (&targets.normal, apply),
      (&targets.albedo, apply),
    ]
    .into_iter()
    .enumerate()
    {
      let mut pass: wgpu::RenderPass<'_> = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("wet"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
          view: target,
          depth_slice: None,
          resolve_target: None,
          ops: wgpu::Operations {
            load: wgpu::LoadOp::Load,
            store: wgpu::StoreOp::Store,
          },
        })],
        ..Default::default()
      });

      pass.set_pipeline(&self.pipelines[index]);
      pass.set_bind_group(0, &view.bind_group, &[]);
      pass.set_bind_group(1, group, &[]);
      pass.draw(0..3, 0..1);
    }
  }

  fn create_pipelines(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    view_layout: &wgpu::BindGroupLayout,
    [patch, apply]: &[wgpu::BindGroupLayout; 2],
  ) -> XrfResult<[wgpu::RenderPipeline; 3]> {
    let patched: wgpu::RenderPipeline = create_fullscreen_pipeline(
      device,
      shaders,
      "frame/wet_patch",
      "fs_wet_patch",
      &[Some(view_layout), Some(patch)],
      ViewTargets::LIGHT,
    )?;
    let normal: wgpu::RenderPipeline = create_fullscreen_pipeline(
      device,
      shaders,
      "frame/wet_apply",
      "fs_wet_normal",
      &[Some(view_layout), Some(apply)],
      ViewTargets::NORMAL,
    )?;
    // `blend(zero, srccolor)` on colour and `(one, one)` on alpha: the albedo multiplied, the gloss added.
    let gloss: wgpu::RenderPipeline = create_fullscreen_pipeline(
      device,
      shaders,
      "frame/wet_apply",
      "fs_wet_gloss",
      &[Some(view_layout), Some(apply)],
      wgpu::ColorTargetState {
        format: ViewTargets::ALBEDO,
        blend: Some(wgpu::BlendState {
          color: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::Zero,
            dst_factor: wgpu::BlendFactor::Src,
            operation: wgpu::BlendOperation::Add,
          },
          alpha: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::One,
            operation: wgpu::BlendOperation::Add,
          },
        }),
        write_mask: wgpu::ColorWrites::ALL,
      },
    )?;

    Ok([patched, normal, gloss])
  }
}
