//! A tree type baked into its impostor's atlases (`LevelImpostors`): drawn by the G-buffer's own shaders from every
//! frame's direction, then packed, through the frame graph, and read back; kept in the cache, and read from it.

use std::sync::Arc;
use std::time::Duration;

use glam::{UVec2, Vec3};
use xrf_renderer_core::{FrameGraph, GraphBindings, GraphCompileOptions, GraphRuntime, PipelineCache, ProxyAllocator};

use crate::contract::render_impostor_mode::RenderImpostorMode;
use crate::contract::render_lod_settings::RenderLodSettings;
use crate::frame::depth_pyramid::DepthPyramid;
use crate::frame::frame_encoder::FrameEncoder;
use crate::frame::static_scene_handles::StaticSceneHandles;
use crate::host::render_asset_source::RenderAssetSource;
use crate::host::render_cache_store::RenderCacheStore;
use crate::host::render_level_source::RenderLevelSource;
use crate::host::render_tree_type::RenderTreeType;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::static_cull_uniform::StaticCullUniform;
use crate::pass::static_gbuffer_pass::StaticGBufferPass;
use crate::pass::static_occlusion_uniform::StaticOcclusionUniform;
use crate::pass::view_binding::ViewBinding;
use crate::pass::wind_uniform::WindUniform;
use crate::scene::level::impostor_atlas::ImpostorAtlas;
use crate::scene::level::level_impostors::LevelImpostors;
use crate::scene::static_scene::static_impostor_frames::StaticImpostorFrames;
use crate::scene::static_scene::static_item_context::StaticItemContext;
use crate::scene::static_scene::static_scene::StaticScene;
use crate::scene::static_scene::static_sector_proxy::StaticSectorProxy;
use crate::scene::texture::texture_cache::TextureCache;
use crate::shader::shader_library::ShaderLibrary;
use crate::tests::test_context::create_test_context;
use crate::tests::test_source::{EmptySource, MemoryCacheStore, TreeTypesSource};
use crate::tests::test_workers::create_workers;
use crate::tests::tree_sector::{TREE_KIND, create_tree_sector};
use crate::thread::render_workers::RenderWorkers;

/// Texels a side of a frame the test bakes at.
const FRAME: u32 = 16;

/// The content key the test level gives its types.
const KEY: u64 = 0x0123_4567_89ab_cdef;

/// An atlas's first level read back, its 16-byte blocks row by row.
fn read_atlas(device: &wgpu::Device, queue: &wgpu::Queue, texture: &wgpu::Texture, size: u32) -> Vec<[u8; 16]> {
  // A row of blocks of an atlas the test bakes is a multiple of the copy's row alignment already.
  let blocks: u32 = size / 4;
  let row: u32 = blocks * 16;
  let buffer: wgpu::Buffer = device.create_buffer(&wgpu::BufferDescriptor {
    label: Some("impostor atlas readback"),
    size: u64::from(row * blocks),
    usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
    mapped_at_creation: false,
  });
  let mut encoder: wgpu::CommandEncoder = device.create_command_encoder(&Default::default());

  encoder.copy_texture_to_buffer(
    texture.as_image_copy(),
    wgpu::TexelCopyBufferInfo {
      buffer: &buffer,
      layout: wgpu::TexelCopyBufferLayout {
        offset: 0,
        bytes_per_row: Some(row),
        rows_per_image: Some(blocks),
      },
    },
    wgpu::Extent3d {
      width: size,
      height: size,
      depth_or_array_layers: 1,
    },
  );
  queue.submit([encoder.finish()]);
  buffer.slice(..).map_async(wgpu::MapMode::Read, |_| ());
  device.poll(wgpu::PollType::wait_indefinitely()).unwrap();

  bytemuck::cast_slice(&buffer.slice(..).get_mapped_range().unwrap()).to_vec()
}

/// The texel at the middle of a frame.
fn get_middle(frame: UVec2) -> UVec2 {
  frame * FRAME + FRAME / 2
}

/// What a BC3 or BC4 alpha block, a block's first eight bytes, holds at a texel of an atlas `size` texels a side, out of
/// 255: its endpoints, eight steps between them where the first is the greater, and the texel's three-bit index.
fn read_alpha(blocks: &[[u8; 16]], size: u32, at: UVec2) -> u8 {
  let block: &[u8; 16] = &blocks[((at.y / 4) * (size / 4) + at.x / 4) as usize];
  let (first, second): (f32, f32) = (f32::from(block[0]), f32::from(block[1]));
  let mut bits: [u8; 8] = [0; 8];

  bits[..6].copy_from_slice(&block[2..8]);

  let texel: u32 = (at.y % 4) * 4 + at.x % 4;
  let index: u64 = (u64::from_le_bytes(bits) >> (texel * 3)) & 7;
  let value: f32 = match index {
    0 => first,
    1 => second,
    _ if first > second => ((8 - index) as f32 * first + (index - 1) as f32 * second) / 7.0,
    6 => 0.0,
    7 => 255.0,
    _ => ((6 - index) as f32 * first + (index - 1) as f32 * second) / 5.0,
  };

  value.round() as u8
}

/// Runs `step` until it answers true, giving the loader threads and the GPU a moment between tries.
fn wait_until(what: &str, mut step: impl FnMut() -> bool) {
  for _ in 0..2_500 {
    if step() {
      return;
    }

    std::thread::sleep(Duration::from_millis(2));
  }

  panic!("Waited too long for {what}");
}

/// The fixture's tree, a triangle standing upright in its own `z = 0` plane, is baked whole: the frame looking along
/// its face sees it across its middle at the depth of its sphere's middle, a frame's corner beyond its sphere sees
/// nothing, and no frame leaks into the next. The bake is kept, and another level of the same type reads it back
/// whole rather than baking. Skipped, and says so, on a machine with no adapter at all.
#[test]
fn a_tree_type_is_baked_into_its_atlases_from_every_frame_and_kept() {
  let Some(context) = create_test_context("to bake with") else {
    return;
  };
  let (device, queue): (&wgpu::Device, &wgpu::Queue) = (&context.device, &context.queue);
  let scope: wgpu::ErrorScopeGuard = device.push_error_scope(wgpu::ErrorFilter::Validation);
  let shaders: ShaderLibrary = ShaderLibrary::default();
  let cache: PipelineCache = PipelineCache::new();
  let source: PipelineSource<'_> = PipelineSource {
    cache: &cache,
    device,
    shaders: &shaders,
  };
  let workers: RenderWorkers = create_workers();
  let assets: Arc<dyn RenderAssetSource> = Arc::new(EmptySource);
  let level: Arc<dyn RenderLevelSource> = Arc::new(TreeTypesSource {
    types: vec![RenderTreeType { trees: 1, key: KEY }; TREE_KIND as usize + 1],
  });
  let store: Arc<MemoryCacheStore> = Arc::new(MemoryCacheStore::default());
  let mut textures: TextureCache = TextureCache::new(device, queue, &workers);
  let view_layout: wgpu::BindGroupLayout = ViewBinding::create_layout(device);
  let gbuffer: StaticGBufferPass = StaticGBufferPass::new(source, &view_layout, textures.get_layout()).unwrap();
  let mut scene: StaticScene = StaticScene::new(device, queue);
  let mut sectors: ProxyAllocator<StaticSectorProxy> = ProxyAllocator::new();
  let mut encoder: FrameEncoder = FrameEncoder::new(device, FrameEncoder::create_belt(device));
  let new_impostors = || {
    LevelImpostors::new(
      device,
      &view_layout,
      (Some(Arc::clone(&store) as Arc<dyn RenderCacheStore>), &workers),
    )
  };
  let mut impostors: LevelImpostors = new_impostors();
  let size: u32 = FRAME * StaticImpostorFrames::FRAMES;
  let settings: RenderLodSettings = RenderLodSettings {
    impostor_mode: RenderImpostorMode::Baked,
    impostor_size: size,
    impostor_share: 1.0,
    ..RenderLodSettings::default()
  };

  scene
    .add_sector(
      &mut StaticItemContext {
        device,
        queue,
        encoder: &mut encoder,
        textures: &mut textures,
        source: &assets,
      },
      &[],
      (sectors.allocate(), &create_tree_sector()),
    )
    .unwrap();
  scene.prepare_draws(device, queue, &mut encoder);
  impostors.follow(&settings);

  // The types are read, the cache asked and found empty, and the type baked.
  wait_until("the bake", || {
    impostors.advance((device, queue), (&scene, &level), true);
    impostors.prepare((device, &mut encoder, source), &scene);
    impostors.get_atlas(TREE_KIND).is_some()
  });

  assert_eq!(impostors.count_ready(&scene), Some((1, 1)));

  queue.submit([encoder.finish().0]);

  let pyramid: DepthPyramid = DepthPyramid::new(device, 4, 4);
  let mut runtime: GraphRuntime = GraphRuntime::new(device, queue);

  {
    let mut graph: FrameGraph<'_> = FrameGraph::new();
    let mut bindings: GraphBindings<'_> = GraphBindings::new();
    let handles: StaticSceneHandles = StaticSceneHandles::import(
      (&mut graph, &mut bindings, &mut runtime),
      &scene,
      &pyramid.view,
      (
        &<StaticCullUniform as bytemuck::Zeroable>::zeroed(),
        &<StaticOcclusionUniform as bytemuck::Zeroable>::zeroed(),
        &WindUniform::default(),
      ),
    );

    impostors.add_passes(
      (&mut graph, &mut bindings, &mut runtime),
      &gbuffer,
      &handles,
      textures.get_bind_group(),
    );

    let commands: Vec<wgpu::CommandBuffer> = graph
      .compile(&GraphCompileOptions::default())
      .unwrap()
      .execute((device, queue), &mut runtime, &bindings)
      .unwrap()
      .commands;

    queue.submit(commands);
  }

  impostors.request_saves();
  wait_until("the bake to be kept", || {
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    impostors.advance((device, queue), (&scene, &level), true);
    !store.entries.lock().unwrap().is_empty()
  });

  let atlas: &ImpostorAtlas = impostors.get_atlas(TREE_KIND).expect("the type's atlases");

  assert_eq!((atlas.size, atlas.levels), (size, ImpostorAtlas::get_levels(FRAME)));
  assert_eq!(atlas.levels, 3, "down to frames four texels across");

  let colour: Vec<[u8; 16]> = read_atlas(device, queue, &atlas.textures[0], size);
  let material: Vec<[u8; 16]> = read_atlas(device, queue, &atlas.textures[2], size);

  assert!(pollster::block_on(scope.pop()).is_none(), "the bake validates");

  // The frames looking along the triangle's face from either side, one of which faces it.
  let faces: Vec<UVec2> = [Vec3::Z, Vec3::NEG_Z]
    .map(|direction| {
      let at = StaticImpostorFrames::encode(direction) * (StaticImpostorFrames::FRAMES - 1) as f32;

      get_middle(at.round().as_uvec2())
    })
    .into_iter()
    .filter(|texel| read_alpha(&colour, size, *texel) == 255)
    .collect();

  assert_eq!(faces.len(), 1, "one side of the triangle faces its frame");

  let depth: u8 = read_alpha(&material, size, faces[0]);

  assert!(
    (f32::from(depth) / 255.0 - 0.5).abs() < 0.1,
    "the face stands across the sphere's middle: {depth}"
  );
  assert_eq!(
    read_alpha(&colour, size, UVec2::ZERO),
    0,
    "the first frame's corner is beyond the sphere"
  );

  // Another level of the type reads it from the cache, before anything of that level is resident, and bakes nothing.
  let normal: Vec<[u8; 16]> = read_atlas(device, queue, &atlas.textures[1], size);
  let scope: wgpu::ErrorScopeGuard = device.push_error_scope(wgpu::ErrorFilter::Validation);
  let mut read: LevelImpostors = new_impostors();

  read.follow(&settings);
  wait_until("the kept bake", || {
    read.advance((device, queue), (&scene, &level), false);
    read.get_atlas(TREE_KIND).is_some()
  });

  let kept: &ImpostorAtlas = read.get_atlas(TREE_KIND).unwrap();

  assert_eq!((kept.size, kept.levels), (atlas.size, atlas.levels));
  assert!(
    [colour, normal, material]
      .iter()
      .zip(&kept.textures)
      .all(|(baked, texture)| *baked == read_atlas(device, queue, texture, size)),
    "the kept atlases are the baked ones"
  );
  assert!(pollster::block_on(scope.pop()).is_none(), "the read validates");
}
