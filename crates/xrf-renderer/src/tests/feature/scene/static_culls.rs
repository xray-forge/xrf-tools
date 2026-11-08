use std::sync::Arc;

use glam::{Mat4, Vec3, Vec4};
use xrf_renderer_core::{
  FrameGraph, GraphBindings, GraphCompileOptions, GraphDeclaration, GraphRuntime, ProxyAllocator,
};
use xrf_visual::SectorSurface;

use crate::camera::camera_view::CameraView;
use crate::contract::view::render_view_options::RenderViewOptions;
use crate::contract::viewport::render_rect::RenderRect;
use crate::feature::scene::statics::statics_feature::StaticsFeature;
use crate::frame::frame_encoder::FrameEncoder;
use crate::frame::static_scene_handles::StaticSceneHandles;
use crate::frame::targets::view_target_handles::ViewTargetHandles;
use crate::frame::targets::view_target_uses::ViewTargetUses;
use crate::frame::targets::view_targets::ViewTargets;
use crate::host::level::spawn::model::static_model_place::StaticModelPlace;
use crate::host::render_asset_source::RenderAssetSource;
use crate::pass::level_passes::LevelPasses;
use crate::pass::view::camera_uniform::CameraUniform;
use crate::pass::view::view_binding::ViewBinding;
use crate::scene::level::view::record::lighting_handles::LightingHandles;
use crate::scene::level::view::record::view_record::ViewRecord;
use crate::scene::static_scene::draw::static_cull_scratch::StaticCullScratch;
use crate::scene::static_scene::geometry::vertex::static_layout::StaticLayout;
use crate::scene::static_scene::item::model::static_model::StaticModel;
use crate::scene::static_scene::item::model::static_model_part::StaticModelPart;
use crate::scene::static_scene::item::static_item_context::StaticItemContext;
use crate::scene::static_scene::static_scene::StaticScene;
use crate::shader::shader_library::ShaderLibrary;
use crate::tests::support::test_bundle::TestBundle;
use crate::tests::support::test_context::create_test_context;
use crate::tests::support::test_source::EmptySource;
use crate::tests::support::test_workers::create_workers;
use crate::thread::gpu_state::GpuState;

const WIDTH: u32 = 256;
const HEIGHT: u32 = 192;

/// A model of one triangle in one cluster a metre about its origin.
fn create_model() -> StaticModel {
  StaticModel {
    words: vec![0; StaticLayout::STRIDE as usize * 3],
    indices: vec![0, 1, 2],
    parts: vec![StaticModelPart {
      indices: (0, 3),
      clusters: vec![(0, 1, Vec4::new(0.0, 0.0, 0.0, 1.0))],
      surface: SectorSurface {
        shader_id: 0,
        shader_name: Some(String::from("models/part")),
        texture_name: Some(String::from("test")),
        hemi: None,
      },
      descriptor: None,
    }],
    sphere: Vec4::new(0.0, 0.0, 0.0, 1.0),
    bounds: [Vec3::splat(-1.0), Vec3::ONE],
    skin: None,
  }
}

/// A view of the scene: its camera, binding, targets and statics.
struct TestView {
  camera: CameraView,
  binding: ViewBinding,
  targets: ViewTargets,
  statics: StaticsFeature,
}

impl TestView {
  /// A view from the origin looking along `look`.
  fn new(gpu: &GpuState, look: Vec3) -> Self {
    let device: &wgpu::Device = &gpu.context.device;
    let mut statics: StaticsFeature = StaticsFeature::new(device);

    statics.fit(device, (WIDTH, HEIGHT));

    Self {
      camera: CameraView::new(
        Vec3::ZERO,
        glam::camera::rh::view::look_at_mat4(Vec3::ZERO, look, Vec3::Y),
        60.0,
        WIDTH as f32 / HEIGHT as f32,
        0.1,
        1000.0,
      ),
      binding: ViewBinding::new(device, &gpu.view_layout),
      targets: ViewTargets::new(device, WIDTH, HEIGHT),
      statics,
    }
  }

  /// Readies its cull of `scene` and writes its camera.
  fn prepare(&mut self, scene: &StaticScene, encoder: &mut FrameEncoder) {
    self
      .statics
      .prepare(scene, &RenderViewOptions::default(), &self.camera, (WIDTH, HEIGHT));
    self.binding.write(
      encoder,
      &CameraUniform::new(
        &self.camera,
        RenderRect {
          x: 0,
          y: 0,
          width: WIDTH,
          height: HEIGHT,
        },
        Vec4::ONE,
      ),
    );
  }
}

/// Culls and draws `scene` through each of `views` in turn in one frame's graph, as a frame draws the viewports showing
/// one level, and answers what the camera's arguments count once the last view culled: its clusters and triangles kept,
/// then occluded.
fn draw_views(
  gpu: &GpuState,
  shaders: &ShaderLibrary,
  scene: &mut StaticScene,
  views: &mut [&mut TestView],
) -> [u32; 4] {
  let (device, queue): (&wgpu::Device, &wgpu::Queue) = (&gpu.context.device, &gpu.context.queue);
  let mut encoder: FrameEncoder = FrameEncoder::new(device, FrameEncoder::create_belt(device));

  // The scene readies its draws once a frame, as `LevelScene::advance` does, before any view culls it.
  scene.prepare_draws(device, queue, &mut encoder);

  let scene: &StaticScene = scene;

  for view in views.iter_mut() {
    view.prepare(scene, &mut encoder);
  }

  queue.submit([encoder.finish().0]);

  let passes: LevelPasses<'_> = gpu.get_level_passes(shaders);
  let exposure: wgpu::Buffer = device.create_buffer(&wgpu::BufferDescriptor {
    label: Some("static culls test exposure"),
    size: 256,
    usage: wgpu::BufferUsages::STORAGE,
    mapped_at_creation: false,
  });
  let mut runtime: GraphRuntime = GraphRuntime::new(device, queue);

  {
    let mut graph: FrameGraph<'_> = FrameGraph::new();
    let mut bindings: GraphBindings<'_> = GraphBindings::new();

    for view in views.iter() {
      let statics: StaticSceneHandles = view
        .statics
        .import(
          GraphDeclaration::new(&mut graph, &mut bindings, &mut runtime),
          scene,
          &bytemuck::Zeroable::zeroed(),
        )
        .expect("the view's pyramid is made");
      let handles: ViewTargetHandles = ViewTargetHandles::import(
        &mut graph,
        &mut bindings,
        &view.targets,
        ViewTargetUses::default(),
        &view.targets.light,
      );
      let none = bindings.import_view(&mut graph, "static culls test none", &view.targets.light);
      let lit: LightingHandles<'_> = LightingHandles {
        lighting: runtime.push_uniform(&bytemuck::Zeroable::zeroed()),
        exposure: bindings.import_buffer(&mut graph, "static culls test exposure", &exposure),
        material_lut: bindings.import_view(&mut graph, "material table", &passes.lighting.table.view),
        brdf_table: bindings.import_view(&mut graph, "brdf table", &passes.lighting.brdf_table.view),
        lut_sampler: &passes.lighting.table.sampler,
        shadow_maps: bindings.import_view(&mut graph, "sun shadow maps", passes.lighting.sun.get_unshadowed()),
        shadows: runtime.push_uniform(&bytemuck::Zeroable::zeroed()),
      };

      view.statics.add_gbuffer_passes(
        GraphDeclaration::new(&mut graph, &mut bindings, &mut runtime),
        ViewRecord {
          passes,
          view: &view.binding,
          textures: gpu.textures.get_bind_group(),
          handles,
          statics,
          cull: view.statics.get_cull(),
          lit,
          unlit: none,
          untraced: none,
          no_ends: none,
        },
      );
    }

    let commands: Vec<wgpu::CommandBuffer> = graph
      .compile(&GraphCompileOptions::default())
      .unwrap()
      .execute((device, queue), &mut runtime, &bindings)
      .unwrap()
      .commands;

    queue.submit(commands);
  }

  let readback: wgpu::Buffer = device.create_buffer(&wgpu::BufferDescriptor {
    label: Some("static culls test counts"),
    size: 16,
    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
    mapped_at_creation: false,
  });
  let mut encoder: wgpu::CommandEncoder = device.create_command_encoder(&Default::default());

  encoder.copy_buffer_to_buffer(&scene.cull.args, StaticCullScratch::STATS_OFFSET, &readback, 0, 16);
  queue.submit([encoder.finish()]);
  readback.slice(..).map_async(wgpu::MapMode::Read, |_| ());
  device.poll(wgpu::PollType::wait_indefinitely()).unwrap();

  let counts: [u32; 4] = bytemuck::pod_read_unaligned(&readback.slice(..).get_mapped_range().unwrap());

  counts
}

/// Two viewports showing one level cull into the scene's one set of camera arguments: the second view's cull counts
/// what it sees alone, not what the first saw as well. Skipped, and says so, on a machine with no adapter at all.
#[test]
fn a_second_view_of_a_shared_scene_counts_its_own_cull_alone() {
  let Some(context) = create_test_context("to draw with") else {
    return;
  };
  let shaders: ShaderLibrary = ShaderLibrary::default();
  let mut gpu: GpuState = GpuState::new(context, &shaders, &create_workers(), &TestBundle::checkout(), None).unwrap();
  let scope: wgpu::ErrorScopeGuard = gpu.context.device.push_error_scope(wgpu::ErrorFilter::Validation);
  let source: Arc<dyn RenderAssetSource> = Arc::new(EmptySource);
  let mut scene: StaticScene = StaticScene::new(&gpu.context.device, &gpu.context.queue);
  let mut models = ProxyAllocator::new();
  let mut objects = ProxyAllocator::new();
  let model = models.allocate();

  {
    let GpuState { context, textures, .. } = &mut gpu;
    let (device, queue): (&wgpu::Device, &wgpu::Queue) = (&context.device, &context.queue);
    let mut encoder: FrameEncoder = FrameEncoder::new(device, FrameEncoder::create_belt(device));

    scene
      .add_model(
        &mut StaticItemContext {
          device,
          queue,
          encoder: &mut encoder,
          textures,
          source: &source,
        },
        model,
        &create_model(),
      )
      .unwrap();

    // Three models standing ahead along `x`, which a view looking along `x` sees and one looking back does not.
    for object in 1..=3u32 {
      let place: StaticModelPlace = StaticModelPlace {
        object,
        transform: Mat4::from_translation(Vec3::new(object as f32 * 3.0, 0.0, 0.0)),
        lighting: None,
        group: 0,
      };

      scene
        .add_object(device, queue, objects.allocate(), model, &place)
        .unwrap();
    }

    scene.prepare_draws(device, queue, &mut encoder);
    queue.submit([encoder.finish().0]);
  }

  let mut ahead: TestView = TestView::new(&gpu, Vec3::X);
  let mut behind: TestView = TestView::new(&gpu, Vec3::NEG_X);

  let seen_ahead: [u32; 4] = draw_views(&gpu, &shaders, &mut scene, &mut [&mut ahead]);
  let seen_behind: [u32; 4] = draw_views(&gpu, &shaders, &mut scene, &mut [&mut behind]);
  let seen_after: [u32; 4] = draw_views(&gpu, &shaders, &mut scene, &mut [&mut ahead, &mut behind]);

  assert!(seen_ahead[0] > 0, "the view ahead sees the models: {seen_ahead:?}");
  assert_eq!(seen_behind[0], 0, "the view behind sees none: {seen_behind:?}");
  assert_eq!(seen_after, seen_behind, "the second view counts its own cull alone");

  gpu.context.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();

  assert!(pollster::block_on(scope.pop()).is_none());
}
