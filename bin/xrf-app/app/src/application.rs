//! The composition root: what the application is assembled from, and what is in place before its first command runs.

use std::process::ExitCode;
use std::sync::Arc;

use tauri::{App, Builder, Manager, RunEvent, Runtime, Window, WindowEvent, Wry};
use xrf_app_core::assets::AssetMountState;
use xrf_app_core::directories::AppDirectories;
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::jobs::JobRegistry;
use xrf_app_core::logging::{ProcessStart, setup_logger};
use xrf_app_core::session::SessionScopes;
use xrf_app_core::transport::TransportServer;
use xrf_app_core::window::{WindowCloseApprovals, WindowHandles};
use xrf_app_system::diagnostics::StartupLog;
use xrf_build_info::{BuildInfo, build_info};
use xrf_job::ExecutionRequest;

use crate::ipc::registry::transport_routes;
use crate::plugins::domain_plugins;
use crate::window::MainWindow;

/// Assemble the application from its plugins and hand control to Tauri, logging from the moment its configuration
/// names where; a failure before any window is said aloud and ends the process with failure.
pub fn run() -> ExitCode {
  let context = tauri::generate_context!();

  // Recorded by this binary's build script, so it describes the binary rather than whichever crate reads it.
  let build: BuildInfo = build_info!();

  setup_logger(&context.config().identifier, build.version);

  StartupLog::write(&build);

  // One pool for the process, managed before any plugin's setup, since the renderer's loaders run on it too.
  let execution: ExecutionState = match ExecutionState::new(ExecutionRequest::Auto) {
    Ok(execution) => execution,
    Err(error) => return fail_to_start(&format!("Cannot start the application's worker pool: {error}")),
  };

  log::info!(
    "Execution: {} worker(s) ({})",
    execution.get_plan().get_workers(),
    execution.get_plan().get_origin().as_str()
  );

  let directories: AppDirectories = AppDirectories::for_identifier(&context.config().identifier);
  let builder: Builder<Wry> = Builder::default()
    .plugin(tauri_plugin_fs::init())
    .plugin(tauri_plugin_dialog::init())
    .plugin(tauri_plugin_shell::init())
    .plugin(tauri_plugin_store::Builder::default().build());

  log::info!("Registering domain plugins");

  let built: Result<App<Wry>, tauri::Error> = assemble(builder, build, execution, directories)
    .setup(|application| {
      // Before the window, which asks where the transport listens as its page loads.
      TransportServer::start(application, transport_routes())?;
      MainWindow::build(application)?;

      Ok(())
    })
    .build(context);

  // A plugin's setup, the transport or the window failing ends the build here, before anything is on screen.
  let application: App<Wry> = match built {
    Ok(application) => application,
    Err(error) => return fail_to_start(&format!("Cannot start the application: {error}")),
  };

  application.run(|_, event| match event {
    RunEvent::ExitRequested { code, .. } => log::info!("Application exit requested, code {code:?}"),
    // The last line a log gets, so one that ends without it ended without the application exiting.
    RunEvent::Exit => log::info!("Exiting application after {:.1?}", ProcessStart::get_uptime()),
    _ => {}
  });

  ExitCode::SUCCESS
}

/// The domain plugins over the state every one's setup builds on, managed before any of them: which build is running,
/// the pool, the asset state every setup hands the sessions a commit moves along, the scopes every session's handles
/// are released by, where the application writes, the windows viewports and close approvals address, and the jobs. A
/// window that goes releases every scope its pages held.
fn assemble<R: Runtime>(
  builder: Builder<R>,
  build: BuildInfo,
  execution: ExecutionState,
  directories: AppDirectories,
) -> Builder<R> {
  // Closed documents are dropped on the pool, so neither a command nor the event loop waits for a level to go.
  let scopes: SessionScopes = SessionScopes::new(execution.get_pool());
  let builder: Builder<R> = builder
    .manage(build)
    .manage(execution)
    .manage(AssetMountState::new())
    .manage(scopes)
    .manage(directories)
    .manage(WindowHandles::default())
    .manage(WindowCloseApprovals::default())
    .manage(Arc::new(JobRegistry::new()).start_reporting())
    .on_window_event(|window: &Window<R>, event: &WindowEvent| {
      if matches!(event, WindowEvent::Destroyed) {
        let label: &str = window.label();
        let released: usize = window.state::<SessionScopes>().release_window(label);

        log::info!("Window '{label}' went, releasing the {released} opening(s) its pages held");
      }
    });

  domain_plugins().into_iter().fold(builder, Builder::plugin)
}

/// Logs why the application cannot start and tells the user, whom a release build on Windows has no console to tell,
/// only a log file they do not know to open; answers the failure the process ends with.
fn fail_to_start(message: &str) -> ExitCode {
  log::error!("{message}");

  #[cfg(all(windows, not(debug_assertions)))]
  {
    use windows::Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MB_OK, MessageBoxW};
    use windows::core::HSTRING;

    // SAFETY: both strings outlive the call, and it names no owner window.
    unsafe {
      MessageBoxW(
        None,
        &HSTRING::from(message),
        &HSTRING::from("The application cannot start"),
        MB_OK | MB_ICONERROR,
      )
    };
  }

  ExitCode::FAILURE
}

#[cfg(test)]
mod tests {
  use std::num::NonZeroUsize;
  use std::sync::Arc;

  use tauri::test::{MockRuntime, mock_builder, mock_context, noop_assets};
  use tauri::{App, Manager};
  use xrf_app_archives::browse::ArchiveSubjects;
  use xrf_app_configs::ConfigsProjects;
  use xrf_app_core::assets::AssetMountState;
  use xrf_app_core::directories::AppDirectories;
  use xrf_app_core::execution::ExecutionState;
  use xrf_app_core::jobs::JobRegistry;
  use xrf_app_core::session::SessionScopes;
  use xrf_app_core::window::{WindowCloseApprovals, WindowHandles};
  use xrf_app_dialogs::DialogProjects;
  use xrf_app_exports::ExportsProjects;
  use xrf_app_levels::{SelectedLevels, ShownLevels};
  use xrf_app_render::{RenderViewports, Renderer, TextureReloads, ViewportShows, WorldState};
  use xrf_app_spawn::SpawnFiles;
  use xrf_app_sprite_equipment::EquipmentSprites;
  use xrf_app_system::diagnostics::MachineProbeState;
  use xrf_app_textures::browse::TextureBrowses;
  use xrf_app_textures::encoding::TextureComparisons;
  use xrf_app_translations::TranslationProjects;
  use xrf_app_visuals::{VisualBrowses, VisualModels, VisualPoses};
  use xrf_build_info::{BuildInfo, build_info};
  use xrf_job::ExecutionRequest;

  use super::assemble;

  /// A plugin whose setup misses what it builds on fails the build, and a unit no setup manages fails here, so an order
  /// or a unit gone wrong shows before any command asks. The transport and the window, which the running application
  /// sets up after the plugins, are not built here.
  #[test]
  fn every_plugin_sets_up_over_the_state_managed_before_it() {
    let execution: ExecutionState =
      ExecutionState::new(ExecutionRequest::Workers(NonZeroUsize::new(2).expect("two workers"))).expect("pool starts");
    let application: App<MockRuntime> = assemble(mock_builder(), build_info!(), execution, AppDirectories::new(None))
      .build(mock_context(noop_assets()))
      .expect("every plugin to set up");

    assert!(application.try_state::<BuildInfo>().is_some());
    assert!(application.try_state::<ExecutionState>().is_some());
    assert!(application.try_state::<AssetMountState>().is_some());
    assert!(application.try_state::<SessionScopes>().is_some());
    assert!(application.try_state::<AppDirectories>().is_some());
    assert!(application.try_state::<WindowHandles>().is_some());
    assert!(application.try_state::<WindowCloseApprovals>().is_some());
    assert!(application.try_state::<Arc<JobRegistry>>().is_some());
    assert!(application.try_state::<Renderer>().is_some());
    assert!(application.try_state::<WorldState>().is_some());
    assert!(application.try_state::<ViewportShows>().is_some());
    assert!(application.try_state::<RenderViewports>().is_some());
    assert!(application.try_state::<TextureReloads>().is_some());
    assert!(application.try_state::<SelectedLevels>().is_some());
    assert!(application.try_state::<ShownLevels>().is_some());
    assert!(application.try_state::<VisualModels>().is_some());
    assert!(application.try_state::<VisualPoses>().is_some());
    assert!(application.try_state::<VisualBrowses>().is_some());
    assert!(application.try_state::<TextureBrowses>().is_some());
    assert!(application.try_state::<TextureComparisons>().is_some());
    assert!(application.try_state::<ConfigsProjects>().is_some());
    assert!(application.try_state::<TranslationProjects>().is_some());
    assert!(application.try_state::<ArchiveSubjects>().is_some());
    assert!(application.try_state::<ExportsProjects>().is_some());
    assert!(application.try_state::<DialogProjects>().is_some());
    assert!(application.try_state::<SpawnFiles>().is_some());
    assert!(application.try_state::<EquipmentSprites>().is_some());
    assert!(application.try_state::<MachineProbeState>().is_some());

    application.state::<RenderViewports>().shut_down();
  }
}
