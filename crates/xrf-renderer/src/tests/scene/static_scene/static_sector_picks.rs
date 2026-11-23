use std::sync::Arc;

use xrf_renderer_core::{ProxyAllocator, ProxyHandle};

use crate::contract::overlay::render_selection_target::RenderSelectionTarget;
use crate::frame::frame_encoder::FrameEncoder;
use crate::host::render_asset_source::RenderAssetSource;
use crate::scene::static_scene::draw::slot::static_slot_info::StaticSlotInfo;
use crate::scene::static_scene::item::static_item_context::StaticItemContext;
use crate::scene::static_scene::item::static_sector_proxy::StaticSectorProxy;
use crate::scene::static_scene::static_scene::StaticScene;
use crate::scene::texture::texture_cache::TextureCache;
use crate::tests::scene::static_scene::sector_fixture::create_sector;
use crate::tests::support::test_context::create_test_context;
use crate::tests::support::test_source::EmptySource;
use crate::tests::support::test_workers::create_workers;

/// The clusters a selection of a sector's baked geometry of shader table entry zero marks.
fn select(scene: &StaticScene, sector: u32) -> Vec<u32> {
  scene
    .resolve_selection(&RenderSelectionTarget::Surface {
      sector,
      shader_id: 0,
      mesh: None,
      place: None,
    })
    .map(|selection| selection.runs.iter().flat_map(|[first, end]| *first..*end).collect())
    .unwrap_or_default()
}

// A sector put after another stands past it in every buffer: each cluster its selection marks picks back to it, none
// of the first's, and the scene holds the clusters of both.
#[test]
fn picks_each_sector_through_where_it_was_put() {
  let Some(context) = create_test_context("to hold a scene") else {
    return;
  };
  let (device, queue): (&wgpu::Device, &wgpu::Queue) = (&context.device, &context.queue);
  let source: Arc<dyn RenderAssetSource> = Arc::new(EmptySource);
  let mut textures: TextureCache = TextureCache::new(device, queue, &create_workers());
  let mut scene: StaticScene = StaticScene::new(device, queue);
  let mut sectors: ProxyAllocator<StaticSectorProxy> = ProxyAllocator::new();
  let mut encoder: FrameEncoder = FrameEncoder::new(device, FrameEncoder::create_belt(device));

  for sector in [7, 9] {
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
        handle,
        &create_sector(sector),
      )
      .unwrap();
  }

  queue.submit([encoder.finish().0]);

  let (first, second): (Vec<u32>, Vec<u32>) = (select(&scene, 7), select(&scene, 9));

  assert!(!first.is_empty() && !second.is_empty(), "{first:?} {second:?}");
  assert!(
    first.iter().max() < second.iter().min(),
    "the second sector's clusters stand past the first's: {first:?} {second:?}"
  );
  assert_eq!(scene.get_pools().clusters.used as usize, first.len() + second.len());

  for (sector, clusters) in [(7, &first), (9, &second)] {
    for cluster in clusters {
      let picked: Option<(StaticSlotInfo, Option<u32>)> = scene.resolve_pick(*cluster, 0);

      assert_eq!(
        picked.map(|(info, instance)| (info.sector, info.shader_id, instance)),
        Some((sector, 0, None)),
        "cluster {cluster}"
      );
    }
  }

  assert!(select(&scene, 8).is_empty(), "no sector 8 was put");
  assert!(scene.resolve_pick(second.iter().max().unwrap() + 1, 0).is_none());
}
