use xrf_error::XrfResult;
use xrf_renderer_core::{
  FrameGraph, GraphBindings, GraphBuffer, GraphBufferAccess, GraphColorAttachment, GraphTexture,
};

use crate::frame::fsr_targets::FsrTargets;
use crate::frame::view_targets::ViewTargets;
use crate::pass::fsr_groups::FsrGroups;
use crate::pass::fullscreen_pipeline::{buffer_binding, create_fullscreen_pipeline_into, texture_binding};
use crate::pass::layout_entries::{storage_entry, texture_entry, uniform_entry};
use crate::pass::shader_pipelines::{create_checked, create_module};
use crate::shader::shader_library::ShaderLibrary;

/// Drawn texels a side of one workgroup of the depth reconstruction.
const RECONSTRUCT_WORKGROUP: u32 = 8;

/// The layouts FSR 2's stages bind, in their order.
struct FsrLayouts {
  luma_first: wgpu::BindGroupLayout,
  luma_shading: wgpu::BindGroupLayout,
  reconstruct: wgpu::BindGroupLayout,
  dilate: wgpu::BindGroupLayout,
  reactive: wgpu::BindGroupLayout,
  depth_clip: wgpu::BindGroupLayout,
  lock: wgpu::BindGroupLayout,
  accumulate: wgpu::BindGroupLayout,
}

/// The pipelines of FSR 2's stages.
struct FsrPipelines {
  luma_first: wgpu::RenderPipeline,
  luma_shading: wgpu::RenderPipeline,
  reconstruct: wgpu::ComputePipeline,
  dilate: wgpu::RenderPipeline,
  reactive: wgpu::RenderPipeline,
  depth_clip: wgpu::RenderPipeline,
  lock: wgpu::RenderPipeline,
  accumulate: wgpu::RenderPipeline,
}

/// FSR 2.2's upscaler (FidelityFX, AMD, MIT) over a viewport's frame, which is in the display's range: an exposure of
/// one and no tonemap of its own, motion drawn at the render size; FSR 2's stages, each a pass.
pub struct FsrPass {
  layouts: FsrLayouts,
  pipelines: FsrPipelines,
  sampler: wgpu::Sampler,
  generation: u64,
}

impl FsrPass {
  /// # Errors
  ///
  /// Returns an error when a shader does not compose or compile.
  pub fn new(device: &wgpu::Device, shaders: &ShaderLibrary) -> XrfResult<Self> {
    let layouts: FsrLayouts = Self::create_layouts(device);

    Ok(Self {
      pipelines: Self::create_pipelines(device, shaders, &layouts)?,
      layouts,
      sampler: device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("fsr2"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
      }),
      generation: shaders.get_generation(),
    })
  }

  pub fn refresh(&mut self, device: &wgpu::Device, shaders: &ShaderLibrary) {
    if shaders.get_generation() != self.generation {
      self.generation = shaders.get_generation();

      match Self::create_pipelines(device, shaders, &self.layouts) {
        Ok(pipelines) => self.pipelines = pipelines,
        Err(error) => log::error!("FSR 2 rejected, upscaling with the last one: {error}"),
      }
    }
  }

  pub fn create_bind_groups(
    &self,
    device: &wgpu::Device,
    targets: &ViewTargets,
    fsr: &FsrTargets,
    uniform: &wgpu::Buffer,
  ) -> FsrGroups {
    let sampler = |binding: u32| wgpu::BindGroupEntry {
      binding,
      resource: wgpu::BindingResource::Sampler(&self.sampler),
    };
    let create = |label: &str, layout: &wgpu::BindGroupLayout, entries: &[wgpu::BindGroupEntry<'_>]| {
      device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout,
        entries,
      })
    };
    let layouts: &FsrLayouts = &self.layouts;

    FsrGroups {
      luma_first: create(
        "fsr2 luma first",
        &layouts.luma_first,
        &[
          buffer_binding(0, uniform),
          texture_binding(1, &targets.scene),
          sampler(2),
        ],
      ),
      luma_shading: create(
        "fsr2 luma shading",
        &layouts.luma_shading,
        &[buffer_binding(0, uniform), texture_binding(1, &fsr.luma_first)],
      ),
      reconstruct: create(
        "fsr2 reconstruct",
        &layouts.reconstruct,
        &[
          buffer_binding(0, uniform),
          texture_binding(1, &targets.depth),
          texture_binding(2, &targets.motion),
          buffer_binding(3, &fsr.reconstructed),
        ],
      ),
      dilate: create(
        "fsr2 dilate",
        &layouts.dilate,
        &[
          buffer_binding(0, uniform),
          texture_binding(1, &targets.depth),
          texture_binding(2, &targets.motion),
          texture_binding(3, &targets.scene),
        ],
      ),
      reactive: create(
        "fsr2 reactive",
        &layouts.reactive,
        &[
          buffer_binding(0, uniform),
          texture_binding(1, &fsr.opaque),
          texture_binding(2, &targets.scene),
        ],
      ),
      depth_clip: [0, 1].map(|index: usize| {
        create(
          "fsr2 depth clip",
          &layouts.depth_clip,
          &[
            buffer_binding(0, uniform),
            texture_binding(1, &targets.scene),
            texture_binding(2, &targets.motion),
            buffer_binding(3, &fsr.reconstructed),
            texture_binding(4, &fsr.dilated_depth[index]),
            texture_binding(5, &fsr.dilated_motion[index]),
            texture_binding(6, &fsr.dilated_motion[1 - index]),
            texture_binding(7, &fsr.reactive),
            sampler(8),
          ],
        )
      }),
      lock: [0, 1].map(|index: usize| {
        create(
          "fsr2 lock",
          &layouts.lock,
          &[buffer_binding(0, uniform), texture_binding(1, &fsr.lock_luma[index])],
        )
      }),
      accumulate: [0, 1].map(|index: usize| {
        let read: usize = 1 - index;

        create(
          "fsr2 accumulate",
          &layouts.accumulate,
          &[
            buffer_binding(0, uniform),
            texture_binding(1, &fsr.prepared),
            texture_binding(2, &fsr.masks),
            texture_binding(3, &fsr.dilated_motion[index]),
            texture_binding(4, &fsr.locks),
            texture_binding(5, &fsr.luma_shading),
            texture_binding(6, &fsr.history[read]),
            texture_binding(7, &fsr.lock_status[read]),
            texture_binding(8, &fsr.luma_history[read]),
            sampler(9),
          ],
        )
      }),
    }
  }

  /// Declares FSR 2's stages, which leave the upscaled frame in this frame's history: the reconstruction's buffer
  /// cleared (`ClearResourcesForNextFrame`, run before the reconstruction rather than after the lock: the far plane, zero
  /// reversed), the luma, the reconstruction, the dilation, the reactive mask, the depth clip, the locks and the
  /// accumulation, each full-screen stage into its targets cleared first.
  pub fn add_passes<'a>(
    &'a self,
    graph: &mut FrameGraph<'a>,
    bindings: &mut GraphBindings<'a>,
    (fsr, groups): (&'a FsrTargets, &'a FsrGroups),
  ) {
    let index: usize = fsr.index;
    let pipelines: &'a FsrPipelines = &self.pipelines;
    let reconstructed: GraphBuffer = bindings.import_buffer(graph, "fsr2 reconstructed", &fsr.reconstructed);
    let mut import = |label: &'static str, view: &'a wgpu::TextureView| bindings.import_view(graph, label, view);
    let stages: [(
      &'static str,
      Vec<GraphTexture>,
      &'a wgpu::RenderPipeline,
      &'a wgpu::BindGroup,
    ); 7] = [
      (
        "fsr2 luma first",
        vec![import("fsr2 luma first", &fsr.luma_first)],
        &pipelines.luma_first,
        &groups.luma_first,
      ),
      (
        "fsr2 luma shading",
        vec![import("fsr2 luma shading", &fsr.luma_shading)],
        &pipelines.luma_shading,
        &groups.luma_shading,
      ),
      (
        "fsr2 dilate",
        vec![
          import("fsr2 dilated depth", &fsr.dilated_depth[index]),
          import("fsr2 dilated motion", &fsr.dilated_motion[index]),
          import("fsr2 lock luma", &fsr.lock_luma[index]),
        ],
        &pipelines.dilate,
        &groups.dilate,
      ),
      (
        "fsr2 reactive",
        vec![import("fsr2 reactive", &fsr.reactive)],
        &pipelines.reactive,
        &groups.reactive,
      ),
      (
        "fsr2 depth clip",
        vec![import("fsr2 prepared", &fsr.prepared), import("fsr2 masks", &fsr.masks)],
        &pipelines.depth_clip,
        &groups.depth_clip[index],
      ),
      (
        "fsr2 lock",
        vec![import("fsr2 locks", &fsr.locks)],
        &pipelines.lock,
        &groups.lock[index],
      ),
      (
        "fsr2 accumulate",
        vec![
          import("fsr2 history", &fsr.history[index]),
          import("fsr2 lock status", &fsr.lock_status[index]),
          import("fsr2 luma history", &fsr.luma_history[index]),
        ],
        &pipelines.accumulate,
        &groups.accumulate[index],
      ),
    ];
    let add_stage = |graph: &mut FrameGraph<'a>,
                     (name, targets, pipeline, group): (
      &'static str,
      Vec<GraphTexture>,
      &'a wgpu::RenderPipeline,
      &'a wgpu::BindGroup,
    )| {
      targets
        .into_iter()
        .fold(graph.add_raster_pass(name), |builder, target| {
          builder.color(GraphColorAttachment::new(
            target,
            wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
          ))
        })
        .record(move |context| {
          let pass: &mut wgpu::RenderPass<'static> = context.get_pass();

          pass.set_pipeline(pipeline);
          pass.set_bind_group(0, group, &[]);
          pass.draw(0..3, 0..1);
        });
    };
    let mut stages = stages.into_iter();

    graph
      .add_encoder_pass("fsr2 clear")
      .buffer(reconstructed, GraphBufferAccess::CopyDestination)
      .record(move |context| {
        let reconstructed: &wgpu::Buffer = context.get_buffer(reconstructed);

        context.get_encoder().clear_buffer(reconstructed, 0, None);
      });

    for stage in stages.by_ref().take(2) {
      add_stage(graph, stage);
    }

    graph
      .add_compute_pass("fsr2 reconstruct")
      .buffer(reconstructed, GraphBufferAccess::StorageReadWrite)
      .record(move |context| {
        let pass: &mut wgpu::ComputePass<'static> = context.get_pass();

        pass.set_pipeline(&pipelines.reconstruct);
        pass.set_bind_group(0, &groups.reconstruct, &[]);
        pass.dispatch_workgroups(
          fsr.render.0.div_ceil(RECONSTRUCT_WORKGROUP),
          fsr.render.1.div_ceil(RECONSTRUCT_WORKGROUP),
          1,
        );
      });

    for stage in stages {
      add_stage(graph, stage);
    }
  }

  fn create_layouts(device: &wgpu::Device) -> FsrLayouts {
    let fragment: wgpu::ShaderStages = wgpu::ShaderStages::FRAGMENT;
    let compute: wgpu::ShaderStages = wgpu::ShaderStages::COMPUTE;
    let flat: wgpu::TextureViewDimension = wgpu::TextureViewDimension::D2;
    let filtered: wgpu::TextureSampleType = wgpu::TextureSampleType::Float { filterable: true };
    let unfiltered: wgpu::TextureSampleType = wgpu::TextureSampleType::Float { filterable: false };
    let depth: wgpu::TextureSampleType = wgpu::TextureSampleType::Depth;
    let sampler = |binding: u32| wgpu::BindGroupLayoutEntry {
      binding,
      visibility: fragment,
      ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
      count: None,
    };
    let create = |label: &str, entries: &[wgpu::BindGroupLayoutEntry]| {
      device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some(label),
        entries,
      })
    };

    FsrLayouts {
      luma_first: create(
        "fsr2 luma first",
        &[
          uniform_entry(0, fragment),
          texture_entry(1, fragment, filtered, flat),
          sampler(2),
        ],
      ),
      luma_shading: create(
        "fsr2 luma shading",
        &[uniform_entry(0, fragment), texture_entry(1, fragment, unfiltered, flat)],
      ),
      reconstruct: create(
        "fsr2 reconstruct",
        &[
          uniform_entry(0, compute),
          texture_entry(1, compute, depth, flat),
          texture_entry(2, compute, unfiltered, flat),
          storage_entry(3, compute, true),
        ],
      ),
      dilate: create(
        "fsr2 dilate",
        &[
          uniform_entry(0, fragment),
          texture_entry(1, fragment, depth, flat),
          texture_entry(2, fragment, unfiltered, flat),
          texture_entry(3, fragment, unfiltered, flat),
        ],
      ),
      reactive: create(
        "fsr2 reactive",
        &[
          uniform_entry(0, fragment),
          texture_entry(1, fragment, unfiltered, flat),
          texture_entry(2, fragment, unfiltered, flat),
        ],
      ),
      depth_clip: create(
        "fsr2 depth clip",
        &[
          uniform_entry(0, fragment),
          texture_entry(1, fragment, unfiltered, flat),
          texture_entry(2, fragment, unfiltered, flat),
          storage_entry(3, fragment, false),
          texture_entry(4, fragment, unfiltered, flat),
          texture_entry(5, fragment, unfiltered, flat),
          texture_entry(6, fragment, filtered, flat),
          texture_entry(7, fragment, unfiltered, flat),
          sampler(8),
        ],
      ),
      lock: create(
        "fsr2 lock",
        &[uniform_entry(0, fragment), texture_entry(1, fragment, unfiltered, flat)],
      ),
      accumulate: create(
        "fsr2 accumulate",
        &[
          uniform_entry(0, fragment),
          texture_entry(1, fragment, filtered, flat),
          texture_entry(2, fragment, filtered, flat),
          texture_entry(3, fragment, unfiltered, flat),
          texture_entry(4, fragment, unfiltered, flat),
          texture_entry(5, fragment, filtered, flat),
          texture_entry(6, fragment, unfiltered, flat),
          texture_entry(7, fragment, filtered, flat),
          texture_entry(8, fragment, filtered, flat),
          sampler(9),
        ],
      ),
    }
  }

  fn create_pipelines(device: &wgpu::Device, shaders: &ShaderLibrary, layouts: &FsrLayouts) -> XrfResult<FsrPipelines> {
    let draw = |module: &str, fragment: &str, layout: &wgpu::BindGroupLayout, formats: &[wgpu::TextureFormat]| {
      let targets: Vec<Option<wgpu::ColorTargetState>> = formats.iter().map(|format| Some((*format).into())).collect();

      create_fullscreen_pipeline_into(device, shaders, module, fragment, &[Some(layout)], &targets)
    };
    let module: wgpu::ShaderModule = create_module(device, shaders, "frame/fsr/reconstruct")?;
    let reconstruct_layout: wgpu::PipelineLayout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("fsr2 reconstruct"),
      bind_group_layouts: &[Some(&layouts.reconstruct)],
      ..Default::default()
    });

    Ok(FsrPipelines {
      luma_first: draw(
        "frame/fsr/luma_first",
        "fs_luma_first",
        &layouts.luma_first,
        &[FsrTargets::LUMA],
      )?,
      luma_shading: draw(
        "frame/fsr/luma_shading",
        "fs_luma_shading",
        &layouts.luma_shading,
        &[FsrTargets::LUMA],
      )?,
      reconstruct: create_checked(device, "fsr2 reconstruct", || {
        device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
          label: Some("fsr2 reconstruct"),
          layout: Some(&reconstruct_layout),
          module: &module,
          entry_point: Some("cs_reconstruct"),
          compilation_options: Default::default(),
          cache: None,
        })
      })?,
      dilate: draw(
        "frame/fsr/dilate",
        "fs_dilate",
        &layouts.dilate,
        &[FsrTargets::DEPTH, FsrTargets::PAIR, FsrTargets::LUMA],
      )?,
      reactive: draw(
        "frame/fsr/reactive",
        "fs_reactive",
        &layouts.reactive,
        &[FsrTargets::MASK],
      )?,
      depth_clip: draw(
        "frame/fsr/depth_clip",
        "fs_depth_clip",
        &layouts.depth_clip,
        &[FsrTargets::PREPARED, FsrTargets::PAIR],
      )?,
      lock: draw("frame/fsr/lock", "fs_lock", &layouts.lock, &[FsrTargets::MASK])?,
      accumulate: draw(
        "frame/fsr/accumulate",
        "fs_accumulate",
        &layouts.accumulate,
        &[FsrTargets::HISTORY, FsrTargets::PAIR, FsrTargets::LUMA_HISTORY],
      )?,
    })
  }
}
