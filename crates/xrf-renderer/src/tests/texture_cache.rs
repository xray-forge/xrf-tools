use std::collections::HashSet;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use xrf_error::{XrfError, XrfResult};

use crate::context::gpu_context::GpuContext;
use crate::contract::render_texture_group::RenderTextureGroup;
use crate::contract::render_texture_state::RenderTextureState;
use crate::frame::periodic::periodic_scheduler::PeriodicScheduler;
use crate::host::render_asset_source::RenderAssetSource;
use crate::scene::texture::texture_cache::TextureCache;
use crate::scene::texture::texture_role::TextureRole;
use crate::tests::test_context::create_test_context;
use crate::tests::test_workers::{TEST_WORKER_NAME, create_workers};

/// A source holding nothing, whose `slow` texture answers only once the test lets it, and then with a failure.
struct GatedSource {
  gate: Mutex<Receiver<()>>,
}

impl RenderAssetSource for GatedSource {
  fn get_texture_scope(&self) -> &str {
    "test"
  }

  fn read_texture(&self, reference: &str) -> XrfResult<Option<Vec<u8>>> {
    if reference == "slow" {
      let _ = self.gate.lock().unwrap().recv();

      return Err(XrfError::new_not_found_error("too late"));
    }

    Ok(None)
  }
}

fn create_source() -> (Arc<dyn RenderAssetSource>, Sender<()>) {
  let (open, gate) = channel();

  (Arc::new(GatedSource { gate: Mutex::new(gate) }), open)
}

/// Updates the cache until a slot's texture is no longer loading.
fn settle(cache: &mut TextureCache, context: &GpuContext, slot: u32) -> RenderTextureState {
  let started: Instant = Instant::now();

  loop {
    cache.update((&context.device, &context.queue), &mut PeriodicScheduler::new(), true);

    let state: RenderTextureState = cache.describe(&[slot], RenderTextureGroup::Surfaces).remove(0).state;

    if state != RenderTextureState::Loading || started.elapsed() > Duration::from_secs(5) {
      return state;
    }

    std::thread::sleep(Duration::from_millis(5));
  }
}

#[test]
fn a_slot_no_scene_samples_is_freed_and_taken_by_the_next_texture() {
  let Some(context) = create_test_context("to upload to") else {
    return;
  };
  let (source, _open) = create_source();
  let mut cache: TextureCache = TextureCache::new(&context.device, &context.queue, &create_workers());
  let kept: u32 = cache.request("kept", TextureRole::Base, &source);
  let dropped: u32 = cache.request("dropped", TextureRole::Base, &source);

  cache.retain(&HashSet::from([kept]));

  assert_eq!(cache.request("kept", TextureRole::Base, &source), kept);
  assert_eq!(cache.request("next", TextureRole::Base, &source), dropped);
  assert_ne!(cache.request("after", TextureRole::Base, &source), dropped);
}

#[test]
fn a_load_landing_after_its_slot_was_freed_is_not_drawn_in_the_slots_next_texture() {
  let Some(context) = create_test_context("to upload to") else {
    return;
  };
  let (source, open) = create_source();
  let mut cache: TextureCache = TextureCache::new(&context.device, &context.queue, &create_workers());
  let slow: u32 = cache.request("slow", TextureRole::Base, &source);

  cache.retain(&HashSet::new());

  let fast: u32 = cache.request("fast", TextureRole::Base, &source);

  assert_eq!(fast, slow);
  assert_eq!(settle(&mut cache, &context, fast), RenderTextureState::Missing);

  open.send(()).unwrap();
  std::thread::sleep(Duration::from_millis(50));
  cache.update((&context.device, &context.queue), &mut PeriodicScheduler::new(), true);

  assert_eq!(
    cache.describe(&[fast], RenderTextureGroup::Surfaces).remove(0).state,
    RenderTextureState::Missing
  );
}

#[test]
fn an_environment_no_scene_samples_is_freed_and_its_slot_taken_by_the_next_cube() {
  let Some(context) = create_test_context("to upload to") else {
    return;
  };
  let mut cache: TextureCache = TextureCache::new(&context.device, &context.queue, &create_workers());
  let kept: u32 = cache.request_environment("kept");
  let dropped: u32 = cache.request_environment("dropped");
  let generation: u64 = cache.get_environments_generation();

  cache.retain_environments(&HashSet::from([kept]));

  assert_eq!(cache.get_environment(dropped), None);
  assert_ne!(cache.get_environments_generation(), generation);
  assert_eq!(cache.request_environment("kept"), kept);
  assert_eq!(cache.request_environment("next"), dropped);
  assert_eq!(cache.get_environment(dropped), Some("next"));
}

/// A source that says which thread read it.
struct ThreadTellingSource {
  told: Mutex<Sender<String>>,
}

impl RenderAssetSource for ThreadTellingSource {
  fn get_texture_scope(&self) -> &str {
    "test"
  }

  fn read_texture(&self, _: &str) -> XrfResult<Option<Vec<u8>>> {
    let name: String = std::thread::current().name().unwrap_or_default().to_owned();
    let _ = self.told.lock().unwrap().send(name);

    Ok(None)
  }
}

// The application hands the renderer its pool; a read on rayon's global pool would add a pool the size of the machine.
#[test]
fn reads_a_texture_on_the_pool_it_was_handed() {
  let Some(context) = create_test_context("to upload to") else {
    return;
  };
  let (told, heard) = channel();
  let source: Arc<dyn RenderAssetSource> = Arc::new(ThreadTellingSource { told: Mutex::new(told) });
  let mut cache: TextureCache = TextureCache::new(&context.device, &context.queue, &create_workers());

  cache.request("any", TextureRole::Base, &source);

  let thread: String = heard.recv_timeout(Duration::from_secs(5)).expect("The texture is read");

  assert!(thread.starts_with(TEST_WORKER_NAME), "read on '{thread}'");
}

/// A source holding nothing, counting its reads and what it was told to forget, its files stamped `stamp`.
struct CountingSource {
  reads: Mutex<usize>,
  forgotten: Mutex<Vec<String>>,
  stamp: u64,
}

impl CountingSource {
  fn new(stamp: u64) -> Arc<Self> {
    Arc::new(Self {
      reads: Mutex::new(0),
      forgotten: Mutex::new(Vec::new()),
      stamp,
    })
  }

  fn get_reads(&self) -> usize {
    *self.reads.lock().unwrap()
  }
}

impl RenderAssetSource for CountingSource {
  fn get_texture_scope(&self) -> &str {
    "test"
  }

  fn read_texture(&self, _: &str) -> XrfResult<Option<Vec<u8>>> {
    *self.reads.lock().unwrap() += 1;

    Ok(None)
  }

  fn forget_textures(&self, names: &[String]) {
    self.forgotten.lock().unwrap().extend_from_slice(names);
  }

  fn get_texture_stamp(&self, _: &str) -> Option<u64> {
    Some(self.stamp)
  }
}

/// Updates the cache until `reads` sources' reads come to `count`, or a while passes; answers the count.
fn wait_reads(cache: &mut TextureCache, context: &GpuContext, reads: &dyn Fn() -> usize, count: usize) -> usize {
  let started: Instant = Instant::now();

  while reads() < count && started.elapsed() < Duration::from_secs(5) {
    std::thread::sleep(Duration::from_millis(5));
  }

  // A read past the count, had one come, lands in this time too.
  std::thread::sleep(Duration::from_millis(100));
  cache.update((&context.device, &context.queue), &mut PeriodicScheduler::new(), true);

  reads()
}

#[test]
fn reloads_a_texture_by_its_file_name_and_its_source_forgets_it() {
  let Some(context) = create_test_context("to upload to") else {
    return;
  };
  let counting: Arc<CountingSource> = CountingSource::new(1);
  let source: Arc<dyn RenderAssetSource> = Arc::clone(&counting) as Arc<dyn RenderAssetSource>;
  let mut cache: TextureCache = TextureCache::new(&context.device, &context.queue, &create_workers());
  let slot: u32 = cache.request(r"ston\Ston_01", TextureRole::Base, &source);
  let reads = || counting.get_reads();

  assert_eq!(settle(&mut cache, &context, slot), RenderTextureState::Missing);
  assert_eq!(wait_reads(&mut cache, &context, &reads, 1), 1);

  cache.reload(&[String::from("other")]);
  assert_eq!(wait_reads(&mut cache, &context, &reads, 2), 1);

  cache.reload(&[String::from("ston_01")]);
  assert_eq!(wait_reads(&mut cache, &context, &reads, 2), 2);
  assert_eq!(*counting.forgotten.lock().unwrap(), vec![String::from("ston_01")]);
}

#[test]
fn reloads_a_texture_a_reopened_source_finds_changed_on_disk() {
  let Some(context) = create_test_context("to upload to") else {
    return;
  };
  let first: Arc<CountingSource> = CountingSource::new(1);
  let same: Arc<CountingSource> = CountingSource::new(1);
  let changed: Arc<CountingSource> = CountingSource::new(2);
  let mut cache: TextureCache = TextureCache::new(&context.device, &context.queue, &create_workers());
  let slot: u32 = cache.request(
    "ston_01",
    TextureRole::Base,
    &(Arc::clone(&first) as Arc<dyn RenderAssetSource>),
  );

  assert_eq!(settle(&mut cache, &context, slot), RenderTextureState::Missing);

  // Asked again by a source opened since: read again only where the file's stamp moved.
  cache.request(
    "ston_01",
    TextureRole::Base,
    &(Arc::clone(&same) as Arc<dyn RenderAssetSource>),
  );
  assert_eq!(wait_reads(&mut cache, &context, &|| same.get_reads(), 1), 0);

  cache.request(
    "ston_01",
    TextureRole::Base,
    &(Arc::clone(&changed) as Arc<dyn RenderAssetSource>),
  );
  assert_eq!(wait_reads(&mut cache, &context, &|| changed.get_reads(), 1), 1);
}
