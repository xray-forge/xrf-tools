use xrf_error::XrfResult;

use crate::frame::pick_target::PickTarget;
use crate::frame::view_targets::ViewTargets;
use crate::pass::layout_entries::{storage_entry, uniform_entry};
use crate::pass::shader_pipelines::{create_checked, create_module};
use crate::pass::static_draw_groups::StaticDrawGroups;
use crate::pass::view_binding::ViewBinding;
use crate::scene::static_scene::static_batch::StaticBatch;
use crate::scene::static_scene::static_class::StaticClass;
use crate::scene::static_scene::static_layout::StaticLayout;
use crate::scene::static_scene::static_scene::StaticScene;
use crate::shader::shader_library::ShaderLibrary;

/// Draws the static scene into a viewport's G-buffer: one indirect draw a batch, its visible clusters as instances, then
/// one for the impostors standing in for distant trees.
pub struct StaticGBufferPass {
  layout: wgpu::BindGroupLayout,
  impostor_layout: wgpu::BindGroupLayout,
  view_layout: wgpu::BindGroupLayout,
  texture_layout: wgpu::BindGroupLayout,
  /// One a batch, by its index.
  pipelines: Vec<wgpu::RenderPipeline>,
  /// One a batch, writing what is drawn rather than how it looks.
  pick_pipelines: Vec<wgpu::RenderPipeline>,
  /// The impostors' draw, then their pick.
  impostor_pipelines: [wgpu::RenderPipeline; 2],
  generation: u64,
}

impl StaticGBufferPass {
  pub fn new(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    view_layout: &wgpu::BindGroupLayout,
    texture_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<Self> {
    let stages: wgpu::ShaderStages = wgpu::ShaderStages::VERTEX_FRAGMENT;
    let layout: wgpu::BindGroupLayout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("static draw"),
      entries: &(0..7)
        .map(|binding| storage_entry(binding, stages, false))
        .chain([
          uniform_entry(7, stages),
          storage_entry(8, stages, false),
          storage_entry(9, stages, false),
        ])
        .collect::<Vec<_>>(),
    });

    let impostor_layout: wgpu::BindGroupLayout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
      label: Some("static impostors"),
      entries: &(0..5)
        .map(|binding| storage_entry(binding, stages, false))
        .collect::<Vec<_>>(),
    });
    let (pipelines, pick_pipelines) = Self::create_pipelines(device, shaders, view_layout, &layout, texture_layout)?;
    let impostor_pipelines =
      Self::create_impostor_pipelines(device, shaders, view_layout, &impostor_layout, texture_layout)?;

    Ok(Self {
      pipelines,
      pick_pipelines,
      impostor_pipelines,
      impostor_layout,
      view_layout: view_layout.clone(),
      texture_layout: texture_layout.clone(),
      generation: shaders.get_generation(),
      layout,
    })
  }

  pub fn refresh(&mut self, device: &wgpu::Device, shaders: &ShaderLibrary) {
    if shaders.get_generation() != self.generation {
      self.generation = shaders.get_generation();

      match Self::create_pipelines(device, shaders, &self.view_layout, &self.layout, &self.texture_layout).and_then(
        |pipelines| {
          let impostors = Self::create_impostor_pipelines(
            device,
            shaders,
            &self.view_layout,
            &self.impostor_layout,
            &self.texture_layout,
          )?;

          Ok((pipelines, impostors))
        },
      ) {
        Ok(((pipelines, pick_pipelines), impostor_pipelines)) => {
          self.pipelines = pipelines;
          self.pick_pipelines = pick_pipelines;
          self.impostor_pipelines = impostor_pipelines;
        }
        Err(error) => log::error!("Static G-buffer rejected, drawing with the last one: {error}"),
      }
    }
  }

  /// The scene's buffers as each layout's draws and the impostors' read them.
  pub fn create_bind_groups(&self, device: &wgpu::Device, scene: &StaticScene) -> StaticDrawGroups {
    let layouts: [wgpu::BindGroup; StaticLayout::COUNT] =
      self.create_layout_groups(device, scene, scene.lists.get_buffer().as_entire_buffer_binding());
    let impostors: wgpu::BindGroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
      label: Some("static impostors"),
      layout: &self.impostor_layout,
      entries: &to_entries(&[
        scene.impostors.get_buffer(),
        scene.corners.get_buffer(),
        scene.terms.get_buffer(),
        scene.impostor_list.get_buffer(),
        scene.surfaces.get_buffer(),
      ]),
    });

    StaticDrawGroups { layouts, impostors }
  }

  /// The scene's buffers as each layout's draws read them, by `StaticLayout::get_index`, drawing a view's own list.
  pub fn create_layout_groups(
    &self,
    device: &wgpu::Device,
    scene: &StaticScene,
    lists: wgpu::BufferBinding<'_>,
  ) -> [wgpu::BindGroup; StaticLayout::COUNT] {
    StaticLayout::ALL.map(|layout| {
      let resources: [wgpu::BindingResource<'_>; 10] = [
        scene.clusters.get_buffer().as_entire_binding(),
        scene.slots.get_buffer().as_entire_binding(),
        scene.places.get_buffer().as_entire_binding(),
        scene.surfaces.get_buffer().as_entire_binding(),
        scene.indices.get_buffer().as_entire_binding(),
        wgpu::BindingResource::Buffer(lists.clone()),
        scene.words[layout.get_index()].get_buffer().as_entire_binding(),
        scene.wind.as_entire_binding(),
        scene.skins.get_buffer().as_entire_binding(),
        scene.bones.get_buffer().as_entire_binding(),
      ];

      device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("static draw"),
        layout: &self.layout,
        entries: &resources
          .into_iter()
          .enumerate()
          .map(|(binding, resource)| wgpu::BindGroupEntry {
            binding: binding as u32,
            resource,
          })
          .collect::<Vec<_>>(),
      })
    })
  }

  /// What every layout's draw binds its clusters by, which a shadow's draws share.
  pub fn get_layout(&self) -> &wgpu::BindGroupLayout {
    &self.layout
  }

  /// Draws the visible clusters an argument buffer lists into the G-buffer the pass draws into, and with the frame's
  /// first draw the impostors too: no occlusion sets one aside.
  pub fn record(
    &self,
    pass: &mut wgpu::RenderPass<'_>,
    (view, bind_groups, textures): (&ViewBinding, &StaticDrawGroups, &wgpu::BindGroup),
    args: &wgpu::Buffer,
    is_first: bool,
  ) {
    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.set_bind_group(1, textures, &[]);

    for (batch, pipeline) in StaticBatch::list_deferred().zip(&self.pipelines) {
      pass.set_pipeline(pipeline);
      pass.set_bind_group(2, &bind_groups.layouts[batch.layout.get_index()], &[]);
      pass.draw_indirect(args, batch.get_index() as u64 * 16);
    }

    if is_first {
      pass.set_pipeline(&self.impostor_pipelines[0]);
      pass.set_bind_group(2, &bind_groups.impostors, &[]);
      pass.draw_indirect(args, StaticScene::IMPOSTOR_ARGS_OFFSET);
    }
  }

  /// Draws the frame's visible clusters again into a pick's one texel, through a camera narrowed to it; the caller
  /// copies the texel out.
  pub fn pick(
    &self,
    encoder: &mut wgpu::CommandEncoder,
    target: &PickTarget,
    view: &ViewBinding,
    bind_groups: &StaticDrawGroups,
    textures: &wgpu::BindGroup,
    args: &[&wgpu::Buffer],
  ) {
    let mut pass: wgpu::RenderPass<'_> = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
      label: Some("static pick"),
      color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        view: &target.color,
        depth_slice: None,
        resolve_target: None,
        ops: wgpu::Operations {
          load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
          store: wgpu::StoreOp::Store,
        },
      })],
      depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
        view: &target.depth,
        depth_ops: Some(wgpu::Operations {
          load: wgpu::LoadOp::Clear(0.0),
          store: wgpu::StoreOp::Discard,
        }),
        stencil_ops: None,
      }),
      ..Default::default()
    });

    pass.set_bind_group(0, &view.bind_group, &[]);
    pass.set_bind_group(1, textures, &[]);

    for (batch, pipeline) in StaticBatch::list_deferred().zip(&self.pick_pipelines) {
      pass.set_pipeline(pipeline);
      pass.set_bind_group(2, &bind_groups.layouts[batch.layout.get_index()], &[]);

      for args in args {
        pass.draw_indirect(args, batch.get_index() as u64 * 16);
      }
    }

    if let Some(args) = args.first() {
      pass.set_pipeline(&self.impostor_pipelines[1]);
      pass.set_bind_group(2, &bind_groups.impostors, &[]);
      pass.draw_indirect(args, StaticScene::IMPOSTOR_ARGS_OFFSET);
    }
  }

  /// The impostors' draw and pick: a quad either way round, so neither culls a face.
  fn create_impostor_pipelines(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    view_layout: &wgpu::BindGroupLayout,
    layout: &wgpu::BindGroupLayout,
    texture_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<[wgpu::RenderPipeline; 2]> {
    let module: wgpu::ShaderModule = create_module(device, shaders, "static/impostor")?;
    let pipeline_layout: wgpu::PipelineLayout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("static impostors"),
      bind_group_layouts: &[Some(view_layout), Some(texture_layout), Some(layout)],
      ..Default::default()
    });
    let targets: [Option<wgpu::ColorTargetState>; 4] = [
      Some(ViewTargets::ALBEDO.into()),
      Some(ViewTargets::NORMAL.into()),
      Some(ViewTargets::MATERIAL.into()),
      Some(ViewTargets::MOTION.into()),
    ];
    let pick_targets: [Option<wgpu::ColorTargetState>; 1] = [Some(PickTarget::FORMAT.into())];
    let create = |fragment: &str, targets: &[Option<wgpu::ColorTargetState>]| -> XrfResult<wgpu::RenderPipeline> {
      create_checked(device, "static impostors", || {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
          label: Some("static impostors"),
          layout: Some(&pipeline_layout),
          vertex: wgpu::VertexState {
            module: &module,
            entry_point: Some("vs_impostor"),
            compilation_options: Default::default(),
            buffers: &[],
          },
          fragment: Some(wgpu::FragmentState {
            module: &module,
            entry_point: Some(fragment),
            compilation_options: Default::default(),
            targets,
          }),
          primitive: Default::default(),
          depth_stencil: Some(wgpu::DepthStencilState {
            format: ViewTargets::DEPTH,
            depth_write_enabled: Some(true),
            depth_compare: Some(wgpu::CompareFunction::Greater),
            stencil: Default::default(),
            bias: Default::default(),
          }),
          multisample: Default::default(),
          multiview_mask: None,
          cache: None,
        })
      })
    };

    Ok([
      create("fs_impostor", &targets)?,
      create("fs_pick_impostor", &pick_targets)?,
    ])
  }

  fn create_pipelines(
    device: &wgpu::Device,
    shaders: &ShaderLibrary,
    view_layout: &wgpu::BindGroupLayout,
    layout: &wgpu::BindGroupLayout,
    texture_layout: &wgpu::BindGroupLayout,
  ) -> XrfResult<(Vec<wgpu::RenderPipeline>, Vec<wgpu::RenderPipeline>)> {
    let module: wgpu::ShaderModule = create_module(device, shaders, "static/gbuffer")?;
    let pipeline_layout: wgpu::PipelineLayout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("static g-buffer"),
      bind_group_layouts: &[Some(view_layout), Some(texture_layout), Some(layout)],
      ..Default::default()
    });
    let targets: [Option<wgpu::ColorTargetState>; 4] = [
      Some(ViewTargets::ALBEDO.into()),
      Some(ViewTargets::NORMAL.into()),
      Some(ViewTargets::MATERIAL.into()),
      Some(ViewTargets::MOTION.into()),
    ];
    let pick_targets: [Option<wgpu::ColorTargetState>; 1] = [Some(PickTarget::FORMAT.into())];
    let create = |is_pick: bool| -> XrfResult<Vec<wgpu::RenderPipeline>> {
      StaticBatch::list_deferred()
        .map(|batch| {
          let vertex: &str = batch.layout.get_vertex_entry();
          let fragment: &str = match (batch.class, is_pick) {
            (StaticClass::CutOut, false) => "fs_cut_out",
            (StaticClass::CutOut, true) => "fs_pick_cut_out",
            (_, false) => "fs_opaque",
            (_, true) => "fs_pick_opaque",
          };

          create_checked(device, "static g-buffer", || {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
              label: Some("static g-buffer"),
              layout: Some(&pipeline_layout),
              vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some(vertex),
                compilation_options: Default::default(),
                buffers: &[],
              },
              fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some(fragment),
                compilation_options: Default::default(),
                targets: if is_pick { &pick_targets } else { &targets },
              }),
              // The packer reversed the winding with the z axis it flipped, so front faces wind counter clockwise.
              primitive: wgpu::PrimitiveState {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
              },
              depth_stencil: Some(wgpu::DepthStencilState {
                format: ViewTargets::DEPTH,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Greater),
                stencil: Default::default(),
                bias: Default::default(),
              }),
              multisample: Default::default(),
              multiview_mask: None,
              cache: None,
            })
          })
        })
        .collect()
    };

    Ok((create(false)?, create(true)?))
  }
}

/// Buffers bound in order, from binding zero.
fn to_entries<'a>(buffers: &[&'a wgpu::Buffer]) -> Vec<wgpu::BindGroupEntry<'a>> {
  buffers
    .iter()
    .enumerate()
    .map(|(binding, buffer)| wgpu::BindGroupEntry {
      binding: binding as u32,
      resource: buffer.as_entire_binding(),
    })
    .collect()
}
