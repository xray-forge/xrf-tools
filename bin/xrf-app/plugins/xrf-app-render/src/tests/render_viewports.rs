use std::path::PathBuf;
use std::sync::Arc;

use xrf_environment::WeatherDescriptor;
use xrf_error::{XrfError, XrfResult};
use xrf_material::XraySurfaceDescriptor;
use xrf_renderer::{
  RenderAssetSource, RenderBundle, RenderEventSink, RenderLevelDetails, RenderLevelParticles, RenderLevelSource,
  RenderLevelSpawn, RenderLevelWeather, RenderQueue, RenderSpawnModels, RenderViewportEvent, RenderViewportId,
  RenderWindowHost, RenderWorkers, SurfaceTargetUnsafe,
};
use xrf_visual::{LightsDescription, SectorPackage};
use xrf_world::{WorldEventSink, WorldViewportEvent};

use crate::render_viewports::RenderViewports;
use crate::renderer::Renderer;
use crate::viewport_shows::ViewportShows;
use crate::world_state::WorldState;

/// A bundle holding nothing.
struct EmptyBundle;

impl RenderBundle for EmptyBundle {
  fn read_bundled(&self, path: &str) -> XrfResult<Vec<u8>> {
    Err(XrfError::new_not_found_error(path))
  }

  fn locate_bundled(&self, _: &str) -> Option<PathBuf> {
    None
  }
}

/// A hidden window, which the render thread never draws into, so no GPU starts.
struct HiddenWindow;

// SAFETY: it names no window, and a thread that would present through it panics in `get_surface_target` instead.
unsafe impl RenderWindowHost for HiddenWindow {
  fn get_key(&self) -> u64 {
    1
  }

  fn get_surface_target(&self) -> SurfaceTargetUnsafe {
    panic!("A hidden test window has no surface to draw into");
  }

  fn get_client_size(&self) -> (u32, u32) {
    (0, 0)
  }

  fn is_hidden(&self) -> bool {
    true
  }
}

/// A page that hears nothing.
struct Silence;

impl RenderEventSink for Silence {
  fn send(&self, _: RenderViewportEvent) -> bool {
    true
  }
}

impl WorldEventSink for Silence {
  fn send(&self, _: WorldViewportEvent) {}
}

/// A level holding nothing, which a viewport can be asked to show.
struct EmptyLevel;

impl RenderAssetSource for EmptyLevel {
  fn get_texture_scope(&self) -> &str {
    "test"
  }

  fn read_texture(&self, _: &str) -> XrfResult<Option<Vec<u8>>> {
    Ok(None)
  }
}

impl RenderLevelSource for EmptyLevel {
  fn get_sector_count(&self) -> u32 {
    0
  }

  fn pack_sector(&self, _: u32) -> XrfResult<SectorPackage> {
    Err(XrfError::new_not_found_error("sector"))
  }

  fn get_surfaces(&self) -> &[XraySurfaceDescriptor] {
    &[]
  }

  fn read_lights(&self) -> XrfResult<LightsDescription> {
    Err(XrfError::new_not_found_error("lights"))
  }

  fn read_weather(&self) -> XrfResult<RenderLevelWeather> {
    Err(XrfError::new_not_found_error("weather"))
  }

  fn read_weather_cycle(&self, _: &str) -> XrfResult<Vec<WeatherDescriptor>> {
    Err(XrfError::new_not_found_error("cycle"))
  }

  fn read_spawn(&self) -> XrfResult<RenderLevelSpawn> {
    Err(XrfError::new_not_found_error("spawn"))
  }

  fn read_spawn_models(&self, _: &[String]) -> XrfResult<RenderSpawnModels> {
    Err(XrfError::new_not_found_error("models"))
  }

  fn read_details(&self) -> XrfResult<Option<RenderLevelDetails>> {
    Ok(None)
  }

  fn read_particles(&self) -> XrfResult<Option<RenderLevelParticles>> {
    Ok(None)
  }
}

/// The units a render plugin sets up, over a renderer that starts no GPU while no window is drawn.
struct Units {
  viewports: RenderViewports,
  shows: ViewportShows,
}

fn create_units() -> Units {
  let pool = Arc::new(
    rayon::ThreadPoolBuilder::new()
      .num_threads(2)
      .build()
      .expect("a test pool"),
  );
  let workers: RenderWorkers = RenderWorkers::new(pool);
  let world: WorldState = WorldState::new(workers.clone());
  let renderer: Renderer = Renderer::new(RenderQueue::new(
    workers,
    Arc::new(EmptyBundle),
    world.as_render_world(),
  ));
  let shows: ViewportShows = ViewportShows::new(world.clone());

  Units {
    viewports: RenderViewports::new(renderer, world, shows.clone()),
    shows,
  }
}

fn attach(viewports: &RenderViewports, window: &str) -> RenderViewportId {
  viewports
    .attach(window, Arc::new(HiddenWindow), (Box::new(Silence), Box::new(Silence)))
    .expect("a viewport number")
}

fn level() -> Option<Arc<dyn RenderLevelSource>> {
  Some(Arc::new(EmptyLevel))
}

#[test]
fn shows_only_what_was_asked_last_of_a_viewport() {
  let Units { viewports, shows } = create_units();
  let viewport: RenderViewportId = attach(&viewports, "main");
  let older: u64 = shows.ask(viewport);
  let newer: u64 = shows.ask(viewport);

  assert_eq!(
    shows.show(viewport, older, level()),
    None,
    "an older ask gives way to a newer"
  );
  assert!(shows.show(viewport, newer, level()).is_some());

  viewports.shut_down();
}

// The world would make the viewport again for a show reaching it after its detach, so the ticket is what refuses it.
#[test]
fn refuses_a_show_asked_before_its_viewport_was_detached() {
  let Units { viewports, shows } = create_units();
  let viewport: RenderViewportId = attach(&viewports, "main");
  let ticket: u64 = shows.ask(viewport);

  viewports.detach(viewport);

  assert_eq!(shows.show(viewport, ticket, level()), None);

  viewports.shut_down();
}

#[test]
fn lets_go_only_of_the_viewports_of_a_window_whose_page_went() {
  let Units { viewports, shows } = create_units();
  let gone: RenderViewportId = attach(&viewports, "main");
  let kept: RenderViewportId = attach(&viewports, "viewer");
  let (gone_ticket, kept_ticket): (u64, u64) = (shows.ask(gone), shows.ask(kept));

  assert!(viewports.detach_window("main"));
  assert!(!viewports.detach_window("main"), "nothing is left in it to let go");
  assert_eq!(shows.show(gone, gone_ticket, level()), None);
  assert!(shows.show(kept, kept_ticket, level()).is_some());

  viewports.shut_down();
}
