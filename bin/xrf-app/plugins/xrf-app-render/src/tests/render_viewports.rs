#![expect(
  unsafe_code,
  reason = "implements the renderer's unsafe window-host contract for a hidden test window"
)]

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

use tauri::ipc::Channel;
use xrf_environment::WeatherDescriptor;
use xrf_error::{XrfError, XrfResult};
use xrf_job::testing::create_test_pool;
use xrf_material::XraySurfaceDescriptor;
use xrf_renderer::{
  RenderAssetSource, RenderBundle, RenderCommand, RenderHoldId, RenderLevelDetails, RenderLevelParticles,
  RenderLevelSource, RenderLevelSpawn, RenderQueue, RenderSpawnModels, RenderViewportId, RenderWindowHost,
  RenderWorkers, SurfaceTargetUnsafe,
};
use xrf_visual::{LightsDescription, SectorPackage};
use xrf_world::{WorldLevelSource, WorldLevelWeather};

use crate::renderer::Renderer;
use crate::viewport::{RenderViewports, ViewportShows};
use crate::world::WorldState;

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

  fn get_surfaces(&self) -> &[XraySurfaceDescriptor] {
    &[]
  }
}

impl WorldLevelSource for EmptyLevel {
  fn pack_sector(&self, _: u32) -> XrfResult<SectorPackage> {
    Err(XrfError::new_not_found_error("sector"))
  }

  fn read_lights(&self) -> XrfResult<LightsDescription> {
    Err(XrfError::new_not_found_error("lights"))
  }

  fn read_weather(&self) -> XrfResult<WorldLevelWeather> {
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
  let pool = Arc::new(create_test_pool(2));
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

/// Viewports over a queue that records every command sent instead of starting a render thread, since a hold would
/// start the GPU; and the commands it recorded.
fn create_recording_viewports() -> (RenderViewports, Receiver<RenderCommand>) {
  let pool = Arc::new(create_test_pool(2));
  let workers: RenderWorkers = RenderWorkers::new(pool);
  let world: WorldState = WorldState::new(workers.clone());
  let (queue, commands): (RenderQueue, Receiver<RenderCommand>) =
    RenderQueue::new_recording(workers, Arc::new(EmptyBundle), world.as_render_world());
  let shows: ViewportShows = ViewportShows::new(world.clone());

  (RenderViewports::new(Renderer::new(queue), world, shows), commands)
}

/// The holds taken and released among the commands recorded since the last call, in the order they were sent.
fn list_hold_commands(commands: &Receiver<RenderCommand>) -> Vec<(bool, RenderHoldId)> {
  commands
    .try_iter()
    .filter_map(|command| match command {
      RenderCommand::Hold { id } => Some((true, id)),
      RenderCommand::Release { id } => Some((false, id)),
      _ => None,
    })
    .collect()
}

/// Attaches a viewport to `window` for a page that hears nothing.
fn attach(viewports: &RenderViewports, window: &str) -> RenderViewportId {
  viewports
    .attach(window, Arc::new(HiddenWindow), Channel::new(|_| Ok(())))
    .expect("a viewport number")
}

fn level() -> Option<Arc<dyn WorldLevelSource>> {
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

#[test]
fn releases_a_hold_only_for_the_window_that_took_it() {
  let (viewports, commands): (RenderViewports, Receiver<RenderCommand>) = create_recording_viewports();
  let hold: RenderHoldId = viewports.hold("main").expect("a hold number");

  assert_eq!(list_hold_commands(&commands), [(true, hold)]);

  viewports.release_hold("viewer", hold);

  assert_eq!(
    list_hold_commands(&commands),
    [],
    "another window's page cannot release it"
  );

  viewports.release_hold("main", hold);

  assert_eq!(list_hold_commands(&commands), [(false, hold)]);

  viewports.release_hold("main", hold);

  assert_eq!(list_hold_commands(&commands), [], "a hold is released once");
}

#[test]
fn releases_the_holds_of_a_window_whose_page_went_and_no_other() {
  let (viewports, commands): (RenderViewports, Receiver<RenderCommand>) = create_recording_viewports();
  let gone: RenderHoldId = viewports.hold("main").expect("a hold number");
  let kept: RenderHoldId = viewports.hold("viewer").expect("a hold number");

  list_hold_commands(&commands);

  assert!(!viewports.detach_window("main"), "it drew no viewport");
  assert_eq!(list_hold_commands(&commands), [(false, gone)]);

  viewports.release_hold("main", gone);

  assert_eq!(
    list_hold_commands(&commands),
    [],
    "its page's own release finds nothing left"
  );

  viewports.release_hold("viewer", kept);

  assert_eq!(list_hold_commands(&commands), [(false, kept)]);
}

// Held no longer once the renderer let go, whatever the destroy answers, so a window that failed to go still closes.
#[test]
fn a_window_whose_viewports_drew_is_held_until_released_then_let_go() {
  let Units { viewports, .. } = create_units();
  let (destroyed, heard): (Sender<()>, Receiver<()>) = channel();

  attach(&viewports, "main");

  assert!(
    !viewports.close_window("tool", || {}),
    "a window that drew nothing closes at once"
  );
  assert!(viewports.close_window("main", move || destroyed.send(()).expect("the test listening")));
  assert!(
    viewports.close_window("main", || {}),
    "a window held stays held whoever asks"
  );

  heard
    .recv_timeout(Duration::from_secs(5))
    .expect("the window destroyed once the renderer let go of it");

  assert!(!viewports.is_closing("main"));
  assert!(!viewports.close_window("main", || {}), "nothing is left to hold it for");

  viewports.shut_down();
}
