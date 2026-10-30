//! The tree types a level's sectors give the scene for their bake (`StaticTreeTypes`): one tree of each, its parts'
//! whole-detail clusters and the sphere it fills in its own space.

use std::sync::Arc;

use xrf_renderer_core::{ProxyAllocator, ProxyHandle};

use crate::frame::frame_encoder::FrameEncoder;
use crate::host::render_asset_source::RenderAssetSource;
use crate::scene::static_scene::static_batch::StaticBatch;
use crate::scene::static_scene::static_class::StaticClass;
use crate::scene::static_scene::static_item_context::StaticItemContext;
use crate::scene::static_scene::static_layout::StaticLayout;
use crate::scene::static_scene::static_scene::StaticScene;
use crate::scene::static_scene::static_sector_proxy::StaticSectorProxy;
use crate::scene::static_scene::static_tree_type::StaticTreeType;
use crate::scene::texture::texture_cache::TextureCache;
use crate::tests::test_context::create_test_context;
use crate::tests::test_source::EmptySource;
use crate::tests::test_workers::create_workers;
use crate::tests::tree_sector::{TREE_KIND, create_tree_sector};

#[test]
fn a_sector_gives_its_tree_types_parts_in_their_own_space_until_it_leaves() {
  let Some(context) = create_test_context("to hold a scene") else {
    return;
  };
  let (device, queue): (&wgpu::Device, &wgpu::Queue) = (&context.device, &context.queue);
  let source: Arc<dyn RenderAssetSource> = Arc::new(EmptySource);
  let mut textures: TextureCache = TextureCache::new(device, queue, &create_workers());
  let mut scene: StaticScene = StaticScene::new(device, queue);
  let mut sectors: ProxyAllocator<StaticSectorProxy> = ProxyAllocator::new();
  let mut encoder: FrameEncoder = FrameEncoder::new(device, FrameEncoder::create_belt(device));
  let handle: ProxyHandle<StaticSectorProxy> = sectors.allocate();

  scene
    .add_sector(
      &mut StaticItemContext {
        device,
        queue,
        encoder: &mut encoder,
        textures: &mut textures,
        source: &source,
      },
      &[],
      (handle, &create_tree_sector()),
    )
    .unwrap();

  let given: Vec<(u32, &StaticTreeType)> = scene.get_tree_types().list().collect();

  assert_eq!(given.len(), 1, "the one type the impostor names");

  let (kind, tree): (u32, &StaticTreeType) = given[0];

  assert_eq!(kind, TREE_KIND);
  assert_eq!(tree.sector, handle);
  assert_eq!(tree.parts.len(), 1, "the tree's one part");
  assert_eq!(tree.parts[0].cluster_count, 1);
  assert_eq!(
    tree.parts[0].batch,
    StaticBatch {
      layout: StaticLayout::Tree,
      class: StaticClass::Opaque,
    }
    .get_index()
  );
  // The triangle around its foot rather than where the first tree stands.
  assert!(tree.sphere.truncate().length() < 2.0, "{}", tree.sphere);
  assert!(tree.sphere.w > 0.5 && tree.sphere.w < 3.0, "{}", tree.sphere);

  assert!(scene.remove_sector(handle));
  queue.submit([encoder.finish().0]);

  assert!(scene.get_tree_types().is_empty(), "its parts left with it");
}
