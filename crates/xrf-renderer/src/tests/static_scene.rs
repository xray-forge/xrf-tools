use std::sync::Arc;

use glam::{Mat4, Vec3, Vec4};
use xrf_error::XrfResult;
use xrf_renderer_core::{ProxyAllocator, ProxyHandle};
use xrf_visual::SectorSurface;

use crate::context::gpu_context::GpuContext;
use crate::context::render_backend::RenderBackend;
use crate::contract::render_selection_target::RenderSelectionTarget;
use crate::host::render_asset_source::RenderAssetSource;
use crate::scene::static_scene::static_layout::StaticLayout;
use crate::scene::static_scene::static_model::StaticModel;
use crate::scene::static_scene::static_model_part::StaticModelPart;
use crate::scene::static_scene::static_model_place::StaticModelPlace;
use crate::scene::static_scene::static_model_proxy::StaticModelProxy;
use crate::scene::static_scene::static_object_proxy::StaticObjectProxy;
use crate::scene::static_scene::static_scene::StaticScene;
use crate::scene::texture::texture_cache::TextureCache;
use crate::tests::test_workers::create_workers;

/// A source holding no textures.
struct EmptySource;

impl RenderAssetSource for EmptySource {
  fn get_texture_scope(&self) -> String {
    String::from("test")
  }

  fn read_texture(&self, _: &str) -> XrfResult<Option<Vec<u8>>> {
    Ok(None)
  }
}

/// One triangle a part, each part of its own shader.
fn create_model(parts: usize) -> StaticModel {
  StaticModel {
    words: vec![0; StaticLayout::STRIDE as usize * 3],
    indices: vec![0, 1, 2],
    parts: (0..parts)
      .map(|index| StaticModelPart {
        clusters: vec![(0, 1, Vec4::new(0.0, 0.0, 0.0, 1.0))],
        surface: SectorSurface {
          shader_id: 0,
          shader_name: Some(format!("models/part_{index}")),
          texture_name: Some(String::from("test")),
          hemi: None,
        },
        descriptor: None,
      })
      .collect(),
    sphere: Vec4::new(0.0, 0.0, 0.0, 1.0),
    bounds: [Vec3::splat(-1.0), Vec3::ONE],
    skin: None,
  }
}

fn place(object: u32) -> StaticModelPlace {
  StaticModelPlace {
    object,
    transform: Mat4::from_translation(Vec3::new(object as f32 * 10.0, 0.0, 0.0)),
    lighting: None,
    group: 0,
  }
}

#[test]
fn stands_objects_as_a_model_and_takes_them_out_again() {
  let Ok(context) =
    GpuContext::create_headless(RenderBackend::D3d12).or_else(|_| GpuContext::create_headless(RenderBackend::Vulkan))
  else {
    eprintln!("Skipped: no GPU to hold a scene");

    return;
  };
  let (device, queue): (&wgpu::Device, &wgpu::Queue) = (&context.device, &context.queue);
  let source: Arc<dyn RenderAssetSource> = Arc::new(EmptySource);
  let mut textures: TextureCache = TextureCache::new(device, queue, &create_workers());
  let mut scene: StaticScene = StaticScene::new(device, queue);
  let mut encoder: wgpu::CommandEncoder = device.create_command_encoder(&Default::default());
  let (mut models, mut objects): (ProxyAllocator<StaticModelProxy>, ProxyAllocator<StaticObjectProxy>) =
    (ProxyAllocator::new(), ProxyAllocator::new());
  let model: ProxyHandle<StaticModelProxy> = models.allocate();

  scene
    .add_model(
      device,
      queue,
      &mut encoder,
      (&mut textures, &source),
      (model, &create_model(2)),
    )
    .unwrap();

  let standing: Vec<ProxyHandle<StaticObjectProxy>> = (1..=3)
    .map(|object| {
      let handle: ProxyHandle<StaticObjectProxy> = objects.allocate();

      scene
        .add_object(device, queue, handle, (model, &place(object)))
        .unwrap();

      handle
    })
    .collect();

  scene.prepare_draws(device, queue, &mut encoder);

  assert_eq!(scene.get_row_count(), 6, "a row an object and part");
  assert_eq!(scene.get_list_capacity(), 6);
  assert_eq!(scene.get_object_sphere(2), Some(Vec4::new(20.0, 0.0, 0.0, 1.0)));

  let at: u32 = scene
    .resolve_selection(&RenderSelectionTarget::Spawn { object: 2 })
    .and_then(|selection| selection.place)
    .unwrap();

  assert_eq!(scene.resolve_spawn_pick(at), Some(2));
  assert!(scene.remove_object(standing[1]));
  assert!(!scene.remove_object(standing[1]), "it is gone already");

  scene.prepare_draws(device, queue, &mut encoder);

  assert_eq!(scene.get_row_count(), 4);
  assert_eq!(scene.get_list_capacity(), 4);
  assert_eq!(scene.resolve_spawn_pick(at), None);
  assert!(!scene.has_object(2) && scene.has_object(1) && scene.has_object(3));

  scene
    .add_object(device, queue, objects.allocate(), (model, &place(2)))
    .unwrap();

  assert!(scene.has_object(2), "shown again, it stands again");
  assert!(scene.remove_model(model));

  scene.prepare_draws(device, queue, &mut encoder);
  queue.submit([encoder.finish()]);

  assert_eq!(scene.get_row_count(), 0);
  assert_eq!(scene.get_list_capacity(), 0);
  assert!(
    !(1..=3).any(|object| scene.has_object(object)),
    "its objects went with it"
  );
  assert_eq!(scene.get_pools().clusters.used, 0, "its clusters were given back");
  assert_eq!(scene.get_pools().places.used, 1, "only the origin place is held");
  assert!(
    scene
      .add_object(device, queue, objects.allocate(), (model, &place(4)))
      .is_err()
  );
}
