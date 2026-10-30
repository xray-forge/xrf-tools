use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use glam::{Mat4, UVec2, UVec4, Vec3, Vec4};
use xrf_renderer_core::{
  ComputePipelineDescription, FrameGraph, GraphBindings, GraphBuffer, GraphBufferAccess, GraphColorAttachment,
  GraphDepthAttachment, GraphRuntime, GraphTexture, GraphTextureDescriptor, PassParameters, StorageArray,
  StorageArrayMut,
};

use crate::camera::camera_view::CameraView;
use crate::contract::render_impostor_mode::RenderImpostorMode;
use crate::contract::render_lod_settings::RenderLodSettings;
use crate::contract::render_rect::RenderRect;
use crate::frame::frame_encoder::FrameEncoder;
use crate::frame::static_scene_handles::StaticSceneHandles;
use crate::frame::view_targets::ViewTargets;
use crate::host::render_cache_store::RenderCacheStore;
use crate::host::render_level_source::RenderLevelSource;
use crate::host::render_tree_type::RenderTreeType;
use crate::pass::camera_uniform::CameraUniform;
use crate::pass::impostor_bake_parameters::ImpostorBakeParameters;
use crate::pass::impostor_encode_parameters::ImpostorEncodeParameters;
use crate::pass::impostor_encode_uniform::ImpostorEncodeUniform;
use crate::pass::impostor_reduce_parameters::ImpostorReduceParameters;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::static_draw_parameters::StaticDrawParameters;
use crate::pass::static_draws::StaticDraws;
use crate::pass::static_gbuffer_pass::StaticGBufferPass;
use crate::pass::static_shadow_pass::StaticShadowPass;
use crate::pass::view_binding::ViewBinding;
use crate::pass::wind_uniform::WindUniform;
use crate::scene::level::impostor_atlas::ImpostorAtlas;
use crate::scene::level::impostor_cache::{ImpostorCache, ImpostorLoad};
use crate::scene::level::impostor_scratch::{ImpostorBlockLevel, ImpostorScratch};
use crate::scene::level::impostor_types::ImpostorTypes;
use crate::scene::static_scene::static_batch::StaticBatch;
use crate::scene::static_scene::static_cull_scratch::StaticCullScratch;
use crate::scene::static_scene::static_geometry::StaticGeometry;
use crate::scene::static_scene::static_impostor_frames::StaticImpostorFrames;
use crate::scene::static_scene::static_layout::StaticLayout;
use crate::scene::static_scene::static_scene::StaticScene;
use crate::scene::static_scene::static_tree_type::StaticTreeType;
use crate::thread::render_workers::RenderWorkers;

/// A level's baked tree impostors: each tree type's atlases under baked impostors, read from the cache where a run
/// kept them, or baked a type a frame once the rest of the level is resident and kept. A bake draws one tree of the
/// type at the origin, still and with its vertices' own hemisphere term, through the G-buffer's own shaders from each
/// frame's direction; packs what it drew into three maps; reduces them level by level, weighed by coverage; and
/// compresses each level into the type's atlases.
pub struct LevelImpostors {
  /// A frame's camera each, in `StaticImpostorFrames::list`'s order.
  cameras: Vec<ViewBinding>,
  /// The bake's draw list, grown to the largest type's clusters, and its draw arguments.
  lists: Option<wgpu::Buffer>,
  args: wgpu::Buffer,
  /// Packing, reducing and compressing: their layouts, and their pipelines with the shader library's generation they
  /// were asked of.
  layouts: [wgpu::BindGroupLayout; 3],
  pipelines: Option<(u64, [wgpu::ComputePipeline; 3])>,
  /// The uncompressed maps a size of atlas is baked through, and the blocks a bake compresses into, grown to the
  /// largest size's.
  scratches: BTreeMap<u32, ImpostorScratch>,
  blocks: Option<wgpu::Buffer>,
  /// Each type's atlases, read or baked: the types ready.
  atlases: BTreeMap<u32, ImpostorAtlas>,
  /// The level's types with their trees, which decide each type's size, and their keys, which the cache knows them by.
  types: ImpostorTypes,
  cache: ImpostorCache,
  workers: RenderWorkers,
  /// The types asked of the cache and not yet answered, and those it kept none of, which bake.
  asked: BTreeSet<u32>,
  missed: BTreeSet<u32>,
  /// Whether the rest of the level is resident, every texture settled, which a bake waits for: it draws the tree as
  /// the level will, and what it draws is kept.
  is_resident: bool,
  /// Whether a view has said how it draws impostors yet, and the full frame size and the share (in thousandths) the
  /// atlases are for under baked ones, a change of either readying them again.
  is_followed: bool,
  baked_at: Option<(u32, u32)>,
  /// How many types were read and baked since the settings last changed, and whether that was said.
  readied: (u32, u32),
  is_told: bool,
  /// The type baked this frame and its atlases' texels a side, and the copy of its blocks going to be kept.
  due: Option<(u32, u32)>,
  saving: Option<usize>,
}

impl LevelImpostors {
  /// The bake's version, which the cache keeps each bake under: raised whenever what a bake draws, packs or compresses
  /// changes, so no earlier bake is read again.
  pub const VERSION: u32 = 1;

  /// Invocations a workgroup runs a side, as `shaders/static/impostor_bake.wgsl` declares them.
  const WORKGROUP: u32 = 8;

  /// The switches a bake's frames are drawn with: textured, bumped, at the hemisphere's whole strength.
  const SWITCHES: Vec4 = Vec4::new(1.0, 1.0, 1.0, 0.0);

  /// Impostors keeping their bakes in `store`, reading and writing them on `workers`.
  pub fn new(
    device: &wgpu::Device,
    view_layout: &wgpu::BindGroupLayout,
    (store, workers): (Option<Arc<dyn RenderCacheStore>>, &RenderWorkers),
  ) -> Self {
    Self {
      cameras: StaticImpostorFrames::list()
        .map(|_| ViewBinding::new(device, view_layout))
        .collect(),
      lists: None,
      args: device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("impostor bake"),
        size: StaticCullScratch::ARGS_SIZE,
        usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      }),
      layouts: [
        ImpostorBakeParameters::create_layout(device),
        ImpostorReduceParameters::create_layout(device),
        ImpostorEncodeParameters::create_layout(device),
      ],
      pipelines: None,
      scratches: BTreeMap::new(),
      blocks: None,
      atlases: BTreeMap::new(),
      types: ImpostorTypes::default(),
      cache: ImpostorCache::new(store, workers.clone(), Self::VERSION),
      workers: workers.clone(),
      asked: BTreeSet::new(),
      missed: BTreeSet::new(),
      is_resident: false,
      is_followed: false,
      baked_at: None,
      readied: (0, 0),
      is_told: false,
      due: None,
      saving: None,
    }
  }

  /// Follows a view's settings: under the engine's impostors, lets the atlases go; under baked ones, lets them go where
  /// the quality or the share they were readied at changed, so every type is read or baked again.
  pub fn follow(&mut self, settings: &RenderLodSettings) {
    let baked_at: Option<(u32, u32)> = (settings.impostor_mode == RenderImpostorMode::Baked).then(|| {
      (
        Self::get_full_frame(settings),
        (settings.impostor_share * 1000.0).round() as u32,
      )
    });

    self.is_followed = true;

    if baked_at == self.baked_at {
      return;
    }

    self.baked_at = baked_at;
    self.atlases.clear();
    self.asked.clear();
    self.missed.clear();
    self.scratches.clear();
    self.blocks = None;
    self.readied = (0, 0);
    self.is_told = false;
  }

  /// Takes a frame of the level's under baked impostors: the types read once, each copy of a bake back since kept, the
  /// atlases the cache answered with put on the GPU, and every type a sector gave that is neither ready nor asked for
  /// asked of the cache, or left to bake where it keeps none. `is_resident` says whether the rest of the level is.
  pub fn advance(
    &mut self,
    (device, queue): (&wgpu::Device, &wgpu::Queue),
    (statics, source): (&StaticScene, &Arc<dyn RenderLevelSource>),
    is_resident: bool,
  ) {
    self.is_resident = is_resident;
    self.cache.write_saves();

    if self.baked_at.is_none() {
      self.cache.release_saves();

      return;
    }

    self.types.read(source, &self.workers);

    if !self.types.poll() {
      return;
    }

    let loads: Vec<ImpostorLoad> = self.cache.take_loads().collect();

    for ImpostorLoad { kind, size, blocks } in loads {
      // An answer for an earlier size, asked before the settings changed, answers nothing now.
      if size != self.get_frame_size(kind) * StaticImpostorFrames::FRAMES || !self.asked.remove(&kind) {
        continue;
      }

      match blocks {
        Some(blocks) => {
          let atlas: ImpostorAtlas =
            ImpostorAtlas::new(device, size, ImpostorAtlas::get_levels(ImpostorAtlas::get_frame(size)));

          atlas.write(queue, &blocks);
          self.atlases.insert(kind, atlas);
          self.readied.0 += 1;
        }
        None => {
          self.missed.insert(kind);
        }
      }
    }

    for (kind, _) in statics.get_tree_types().list() {
      if self.atlases.contains_key(&kind) || self.asked.contains(&kind) || self.missed.contains(&kind) {
        continue;
      }

      match self.types.get_key(kind).filter(|_| self.cache.is_kept()) {
        Some(key) => {
          self
            .cache
            .ask((kind, key), self.get_frame_size(kind) * StaticImpostorFrames::FRAMES);
          self.asked.insert(kind);
        }
        None => {
          self.missed.insert(kind);
        }
      }
    }

    // Nothing waits to be read or baked: what a bake goes through is let go until a type waits again.
    if self.asked.is_empty() && self.missed.is_empty() {
      self.lists = None;
      self.scratches.clear();
      self.blocks = None;
      self.cache.release_saves();
    }

    self.tell_ready(statics);
  }

  /// How many of the types the sectors gave are ready, read or baked, and how many they gave: nought of nought under
  /// the engine's impostors, and none before a view said how it draws them.
  pub fn count_ready(&self, statics: &StaticScene) -> Option<(u32, u32)> {
    if !self.is_followed {
      return None;
    }

    if self.baked_at.is_none() {
      return Some((0, 0));
    }

    let types = statics.get_tree_types();
    let ready: usize = types.list().filter(|(kind, _)| self.atlases.contains_key(kind)).count();

    Some((ready as u32, types.len() as u32))
  }

  /// Readies this frame's bake under baked impostors, once the rest of the level is resident: the first type the cache
  /// kept none of, its draw list and arguments, every frame's camera, its scratch maps and block buffer, its atlases,
  /// and the copy of its blocks to be kept.
  pub fn prepare(
    &mut self,
    (device, encoder, pipelines): (&wgpu::Device, &mut FrameEncoder, PipelineSource<'_>),
    statics: &StaticScene,
  ) {
    self.due = None;
    self.saving = None;

    if self.baked_at.is_none() || !self.is_resident || self.missed.is_empty() || !self.ask_pipelines(pipelines) {
      return;
    }

    let Some((kind, tree)) = statics
      .get_tree_types()
      .list()
      .find(|(kind, _)| self.missed.contains(kind))
    else {
      return;
    };
    let frame: u32 = self.get_frame_size(kind);
    let size: u32 = frame * StaticImpostorFrames::FRAMES;
    let levels: u32 = ImpostorAtlas::get_levels(frame);
    let scratch: &ImpostorScratch = self
      .scratches
      .entry(size)
      .or_insert_with(|| ImpostorScratch::new(device, size, levels));
    let bytes: u64 = u64::from(scratch.block_count) * ImpostorAtlas::BLOCK_BYTES;

    if self.blocks.as_ref().is_none_or(|blocks| blocks.size() < bytes) {
      self.blocks = Some(device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("impostor blocks"),
        size: bytes,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
      }));
    }

    self.write_draws(device, encoder, tree, statics.geometry.get_origin());
    self.write_cameras(encoder, tree, frame);
    self.atlases.insert(kind, ImpostorAtlas::new(device, size, levels));
    self.missed.remove(&kind);
    self.readied.1 += 1;
    self.due = Some((kind, size));
    self.saving = self
      .types
      .get_key(kind)
      .and_then(|key| self.cache.prepare_save(device, (key, size), bytes));
  }

  /// Asks for the copies of the bakes recorded with the frame just submitted to be read back.
  pub fn request_saves(&self) {
    self.cache.request();
  }

  /// Undoes this frame's bake, whose frame was never submitted: its type bakes again, and nothing of it is kept.
  pub fn abandon_frame(&mut self) {
    if let Some((kind, _)) = self.due.take() {
      self.atlases.remove(&kind);
      self.missed.insert(kind);
      self.readied.1 -= 1;
    }

    self.saving = None;
    self.cache.abandon();
  }

  /// Says, once, how the types were readied when every type the sectors gave is, the rest of the level resident.
  fn tell_ready(&mut self, statics: &StaticScene) {
    let types = statics.get_tree_types();

    if self.is_told || !self.is_resident || types.is_empty() || self.atlases.len() < types.len() {
      return;
    }

    self.is_told = true;

    log::info!(
      "Readied the impostors of the level's {} tree types, {} read and {} baked: {:.1} MB",
      types.len(),
      self.readied.0,
      self.readied.1,
      self.get_memory() as f64 / 1e6
    );
  }

  /// Declares this frame's bake: the type's frames drawn by `gbuffer` into a G-buffer of the atlases' size, packed into
  /// the scratch maps, reduced level by level, compressed, and copied into the type's atlases.
  pub fn add_passes<'a>(
    &'a self,
    (graph, bindings, runtime): (&mut FrameGraph<'a>, &mut GraphBindings<'a>, &mut GraphRuntime),
    gbuffer: &'a StaticGBufferPass,
    scene: &StaticSceneHandles,
    textures: &'a wgpu::BindGroup,
  ) {
    let (Some((kind, size)), Some(lists), Some(blocks), Some((_, pipelines))) =
      (self.due, &self.lists, &self.blocks, &self.pipelines)
    else {
      return;
    };
    let (Some(atlas), Some(scratch)) = (self.atlases.get(&kind), self.scratches.get(&size)) else {
      return;
    };
    let lists: GraphBuffer = bindings.import_buffer(graph, "impostor bake lists", lists);
    let args: GraphBuffer = bindings.import_buffer(graph, "impostor bake", &self.args);
    // Still: the default wind sways nothing, its foliage motion off.
    let wind = runtime.push_uniform(&WindUniform::default());
    let layouts: [StaticDrawParameters; StaticLayout::COUNT] = scene
      .get_layout_draws(StorageArray::new(lists))
      .map(|layout| StaticDrawParameters { wind, ..layout });
    let draws: StaticDraws = StaticDraws {
      layouts,
      impostors: scene.get_camera_draws().impostors,
    };
    let [albedo, normal, material, motion, relief, depth]: [GraphTexture; 6] = [
      ("impostor bake albedo", ViewTargets::ALBEDO),
      ("impostor bake normal", ViewTargets::NORMAL),
      ("impostor bake material", ViewTargets::MATERIAL),
      ("impostor bake motion", ViewTargets::MOTION),
      ("impostor bake relief", ViewTargets::RELIEF),
      ("impostor bake depth", ViewTargets::DEPTH),
    ]
    .map(|(label, format)| graph.create_texture(GraphTextureDescriptor::new_2d(label, size, size, format)));
    let frame: u32 = size / StaticImpostorFrames::FRAMES;
    let builder = [albedo, normal, material, motion, relief]
      .into_iter()
      .fold(graph.add_raster_pass("impostor bake"), |builder, texture| {
        builder.color(GraphColorAttachment::new(
          texture,
          wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
        ))
      })
      .depth(GraphDepthAttachment::new(depth, wgpu::LoadOp::Clear(0.0)))
      .buffer(args, GraphBufferAccess::Indirect);

    StaticDraws::declare_layouts(builder, &layouts).record(move |context| {
      let args: &wgpu::Buffer = context.get_buffer(args);

      for (camera, at) in self.cameras.iter().zip(StaticImpostorFrames::list()) {
        StaticShadowPass::set_region(context, [at.x * frame, at.y * frame, frame, frame]);
        gbuffer.record(context, (camera, &draws, textures), args, (false, false, false));
      }
    });

    // Each scratch level as its own resource, packed into the first and each reduced into the next.
    let levels: Vec<[GraphTexture; 3]> = scratch
      .levels
      .iter()
      .map(|views| std::array::from_fn(|map| bindings.import_view(&mut *graph, "impostor bake level", &views[map])))
      .collect();
    let pack: ImpostorBakeParameters = ImpostorBakeParameters {
      depth_target: depth,
      albedo_target: albedo,
      normal_target: normal,
      material_target: material,
      colour_level: levels[0][0],
      normal_level: levels[0][1],
      material_level: levels[0][2],
    };

    graph
      .add_compute_pass("impostor pack")
      .parameters(&pack)
      .record(move |context| {
        context.bind(&pack);
        Self::dispatch(context.get_pass(), &pipelines[0], size);
      });

    let reductions: Vec<ImpostorReduceParameters> = levels
      .windows(2)
      .map(|pair| ImpostorReduceParameters {
        colour_source: pair[0][0],
        normal_source: pair[0][1],
        material_source: pair[0][2],
        colour_level: pair[1][0],
        normal_level: pair[1][1],
        material_level: pair[1][2],
      })
      .collect();

    reductions
      .iter()
      .fold(graph.add_compute_pass("impostor reduce"), |builder, level| {
        builder.parameters(level)
      })
      .record(move |context| {
        for (level, parameters) in reductions.iter().enumerate() {
          context.bind(parameters);
          Self::dispatch(context.get_pass(), &pipelines[1], scratch.blocks[level + 1].size);
        }
      });

    let blocks: GraphBuffer = bindings.import_buffer(graph, "impostor blocks", blocks);
    let encodings: Vec<ImpostorEncodeParameters> = levels
      .iter()
      .zip(&scratch.blocks)
      .map(|(level, layout)| ImpostorEncodeParameters {
        colour_source: level[0],
        normal_source: level[1],
        material_source: level[2],
        blocks: StorageArrayMut::new(blocks),
        settings: runtime.push_uniform(&ImpostorEncodeUniform {
          size: UVec4::new(layout.size, layout.blocks, layout.blocks, layout.row),
          starts: UVec4::new(layout.starts[0], layout.starts[1], layout.starts[2], 0),
        }),
      })
      .collect();

    encodings
      .iter()
      .fold(graph.add_compute_pass("impostor encode"), |builder, level| {
        builder.parameters(level)
      })
      .record(move |context| {
        for (parameters, layout) in encodings.iter().zip(&scratch.blocks) {
          context.bind(parameters);
          Self::dispatch(context.get_pass(), &pipelines[2], layout.blocks);
        }
      });

    // The atlases and the copy kept are the level's, outside the graph, which sees no reader of what the copy writes.
    let save: Option<&wgpu::Buffer> = self.saving.map(|index| self.cache.get_save_buffer(index));
    let bytes: u64 = u64::from(scratch.block_count) * ImpostorAtlas::BLOCK_BYTES;

    graph
      .add_encoder_pass("impostor copy")
      .buffer(blocks, GraphBufferAccess::CopySource)
      .keep()
      .record(move |context| {
        let source: &wgpu::Buffer = context.get_buffer(blocks);
        let encoder: &mut wgpu::CommandEncoder = context.get_encoder();

        Self::copy_blocks(encoder, source, &scratch.blocks, atlas);

        if let Some(save) = save {
          encoder.copy_buffer_to_buffer(source, 0, save, 0, bytes);
        }
      });
  }

  /// A type's atlases, once baked.
  pub fn get_atlas(&self, kind: u32) -> Option<&ImpostorAtlas> {
    self.atlases.get(&kind)
  }

  /// Bytes the baked atlases take.
  pub fn get_memory(&self) -> u64 {
    self.atlases.values().map(ImpostorAtlas::get_memory).sum()
  }

  /// Texels a side of a frame of the settings' quality, a multiple of a block so no block reaches across two frames.
  fn get_full_frame(settings: &RenderLodSettings) -> u32 {
    (settings.impostor_size / StaticImpostorFrames::FRAMES / 4 * 4).max(4)
  }

  /// Texels a side of a type's frames under baked impostors: the settings' quality for the most stood types, together
  /// standing the settings' share of the level's trees, half of it for the rest; the settings' quality for all where
  /// the types are unknown.
  fn get_frame_size(&self, kind: u32) -> u32 {
    let Some((full, share)) = self.baked_at else {
      return 0;
    };
    let types: &[RenderTreeType] = self.types.list();
    let total: u64 = types.iter().map(|kind| u64::from(kind.trees)).sum();
    let mut stood: u64 = 0;
    let detailed: usize = types
      .iter()
      .take_while(|kind| {
        let is_short: bool = stood * 1000 < u64::from(share) * total;

        stood += u64::from(kind.trees);

        is_short
      })
      .count();

    if types.is_empty() || (kind as usize) < detailed {
      full
    } else {
      (full / 8 * 4).max(4)
    }
  }

  fn dispatch(pass: &mut wgpu::ComputePass<'static>, pipeline: &wgpu::ComputePipeline, across: u32) {
    pass.set_pipeline(pipeline);
    pass.dispatch_workgroups(across.div_ceil(Self::WORKGROUP), across.div_ceil(Self::WORKGROUP), 1);
  }

  /// Copies each level's blocks of each map from the block buffer into the type's atlases.
  fn copy_blocks(
    encoder: &mut wgpu::CommandEncoder,
    blocks: &wgpu::Buffer,
    layouts: &[ImpostorBlockLevel],
    atlas: &ImpostorAtlas,
  ) {
    for (level, layout) in layouts.iter().enumerate() {
      for (map, texture) in atlas.textures.iter().enumerate() {
        encoder.copy_buffer_to_texture(
          wgpu::TexelCopyBufferInfo {
            buffer: blocks,
            layout: wgpu::TexelCopyBufferLayout {
              offset: u64::from(layout.starts[map]) * ImpostorAtlas::BLOCK_BYTES,
              bytes_per_row: Some(layout.row * ImpostorAtlas::BLOCK_BYTES as u32),
              rows_per_image: Some(layout.blocks),
            },
          },
          wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: level as u32,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
          },
          wgpu::Extent3d {
            width: layout.blocks * 4,
            height: layout.blocks * 4,
            depth_or_array_layers: 1,
          },
        );
      }
    }
  }

  /// Writes the bake's draw list, each part's clusters standing at `origin` under its batch's draw, and the arguments
  /// drawing them; every other batch draws nothing.
  fn write_draws(&mut self, device: &wgpu::Device, encoder: &mut FrameEncoder, tree: &StaticTreeType, origin: u32) {
    let mut batches: BTreeMap<u32, Vec<[u32; 2]>> = BTreeMap::new();

    for part in &tree.parts {
      batches
        .entry(part.batch)
        .or_default()
        .extend((part.first_cluster..part.first_cluster + part.cluster_count).map(|cluster| [cluster, origin]));
    }

    let mut entries: Vec<[u32; 2]> = Vec::new();
    let mut args: Vec<u32> = vec![0; (StaticCullScratch::ARGS_SIZE / 4) as usize];

    for (batch, clusters) in batches {
      let at: usize = (StaticBatch::get_list_args_offset(batch) / 4) as usize;

      args[at..at + 4].copy_from_slice(&[
        StaticGeometry::CLUSTER_VERTICES,
        clusters.len() as u32,
        0,
        entries.len() as u32,
      ]);
      entries.extend(clusters);
    }

    let bytes: u64 = (entries.len().max(1) * size_of::<[u32; 2]>()) as u64;

    if self.lists.as_ref().is_none_or(|lists| lists.size() < bytes) {
      self.lists = Some(device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("impostor bake lists"),
        size: bytes.next_power_of_two(),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
      }));
    }

    if let Some(lists) = &self.lists {
      encoder.write_buffer(lists, 0, bytemuck::cast_slice(&entries));
    }

    encoder.write_buffer(&self.args, 0, bytemuck::cast_slice(&args));
  }

  /// Writes each frame's camera: looking back along its direction at the type's sphere, parallel rays framing it whole,
  /// its depth running across the sphere, drawn into its square of the atlas.
  fn write_cameras(&self, encoder: &mut FrameEncoder, tree: &StaticTreeType, frame: u32) {
    let center: Vec3 = tree.sphere.truncate();
    let radius: f32 = tree.sphere.w.max(0.01);

    for (camera, at) in self.cameras.iter().zip(StaticImpostorFrames::list()) {
      let direction: Vec3 = StaticImpostorFrames::get_direction(at);
      let (_, up): (Vec3, Vec3) = StaticImpostorFrames::get_basis(direction);
      let eye: Vec3 = center + direction * radius;
      let look: Mat4 = glam::camera::rh::view::look_at_mat4(eye, center, up);
      let view: CameraView = CameraView::new_orthographic(eye, look, (radius, radius), 0.0, 2.0 * radius);
      let region: UVec2 = at * frame;

      camera.write(
        encoder,
        &CameraUniform::new(
          &view,
          RenderRect {
            x: region.x as i32,
            y: region.y as i32,
            width: frame,
            height: frame,
          },
          Self::SWITCHES,
        ),
      );
    }
  }

  /// Asks the shader library for the bake's pipelines where it has changed since; answers whether they are at hand.
  fn ask_pipelines(&mut self, source: PipelineSource<'_>) -> bool {
    let generation: u64 = source.get_generation();

    if self.pipelines.as_ref().is_some_and(|(asked, _)| *asked == generation) {
      return true;
    }

    let create = |label: &'static str, entry: &'static str, layout: &wgpu::BindGroupLayout| {
      source.get_compute_pipeline(
        &ComputePipelineDescription::new(label, "static/impostor_bake", entry).with_bind_group_layouts(&[Some(layout)]),
      )
    };
    let [pack, reduce, encode] = &self.layouts;

    match (|| {
      Ok::<_, xrf_error::XrfError>([
        create("impostor pack", "cs_pack", pack)?,
        create("impostor reduce", "cs_reduce", reduce)?,
        create("impostor encode", "cs_encode", encode)?,
      ])
    })() {
      Ok(pipelines) => self.pipelines = Some((generation, pipelines)),
      Err(error) => {
        log::error!("Impostor bake rejected, baking with the last one: {error}");

        if let Some((asked, _)) = &mut self.pipelines {
          *asked = generation;
        }
      }
    }

    self.pipelines.is_some()
  }
}
