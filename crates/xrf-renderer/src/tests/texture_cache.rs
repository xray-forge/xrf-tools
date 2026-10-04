use std::collections::HashSet;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use xrf_error::{XrfError, XrfResult};

use crate::context::gpu_context::GpuContext;
use crate::context::render_backend::RenderBackend;
use crate::contract::render_texture_state::RenderTextureState;
use crate::host::render_asset_source::RenderAssetSource;
use crate::scene::texture::texture_cache::TextureCache;
use crate::scene::texture::texture_role::TextureRole;

/// A source holding nothing, whose `slow` texture answers only once the test lets it, and then with a failure.
struct GatedSource {
  gate: Mutex<Receiver<()>>,
}

impl RenderAssetSource for GatedSource {
  fn get_texture_scope(&self) -> String {
    String::from("test")
  }

  fn read_texture(&self, reference: &str) -> XrfResult<Option<Vec<u8>>> {
    if reference == "slow" {
      let _ = self.gate.lock().unwrap().recv();

      return Err(XrfError::new_not_found_error("too late"));
    }

    Ok(None)
  }
}

fn create_context() -> Option<GpuContext> {
  GpuContext::create_headless(RenderBackend::D3d12)
    .or_else(|_| GpuContext::create_headless(RenderBackend::Vulkan))
    .map_err(|error| eprintln!("Skipped: no GPU to upload to ({error})"))
    .ok()
}

fn create_source() -> (Arc<dyn RenderAssetSource>, Sender<()>) {
  let (open, gate) = channel();

  (Arc::new(GatedSource { gate: Mutex::new(gate) }), open)
}

/// Updates the cache until a slot's texture is no longer loading.
fn settle(cache: &mut TextureCache, context: &GpuContext, slot: u32) -> RenderTextureState {
  let started: Instant = Instant::now();

  loop {
    cache.update(&context.device, &context.queue);

    let state: RenderTextureState = cache.describe(&[slot]).remove(0).state;

    if state != RenderTextureState::Loading || started.elapsed() > Duration::from_secs(5) {
      return state;
    }

    std::thread::sleep(Duration::from_millis(5));
  }
}

#[test]
fn a_slot_no_scene_samples_is_freed_and_taken_by_the_next_texture() {
  let Some(context) = create_context() else {
    return;
  };
  let (source, _open) = create_source();
  let mut cache: TextureCache = TextureCache::new(&context.device, &context.queue);
  let kept: u32 = cache.request("kept", TextureRole::Base, &source);
  let dropped: u32 = cache.request("dropped", TextureRole::Base, &source);

  cache.retain(&HashSet::from([kept]));

  assert_eq!(cache.request("kept", TextureRole::Base, &source), kept);
  assert_eq!(cache.request("next", TextureRole::Base, &source), dropped);
  assert_ne!(cache.request("after", TextureRole::Base, &source), dropped);
}

#[test]
fn a_load_landing_after_its_slot_was_freed_is_not_drawn_in_the_slots_next_texture() {
  let Some(context) = create_context() else {
    return;
  };
  let (source, open) = create_source();
  let mut cache: TextureCache = TextureCache::new(&context.device, &context.queue);
  let slow: u32 = cache.request("slow", TextureRole::Base, &source);

  cache.retain(&HashSet::new());

  let fast: u32 = cache.request("fast", TextureRole::Base, &source);

  assert_eq!(fast, slow);
  assert_eq!(settle(&mut cache, &context, fast), RenderTextureState::Missing);

  open.send(()).unwrap();
  std::thread::sleep(Duration::from_millis(50));
  cache.update(&context.device, &context.queue);

  assert_eq!(cache.describe(&[fast]).remove(0).state, RenderTextureState::Missing);
}
