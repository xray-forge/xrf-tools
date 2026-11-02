/// Expands a crate's command domains into a module per domain, carrying its plugin builder, dispatch handler, plugin
/// name, Specta collection, bindings surface, transport routes and job kinds; and, for the whole crate, `list_routes`,
/// `list_surfaces` and `list_job_kinds`.
///
/// Expanded in the crate that declares the domains, so its command paths and `typescript-bindings` feature are its own.
/// A domain's job kinds are its plugin name, a dot and the kind's own wire name (`archives.pack`), each a constant the
/// frontend's bindings export under the name declared, which by convention leads with the domain (`ARCHIVES_JOB_PACK`):
/// no domain can spell another's kind, and no two constants share a name in the one module the bindings write.
#[macro_export]
macro_rules! define_runtime_domains {
  (
    $(
      $domain:ident => $plugin_name:literal {
        $($command_name:ident => $command_head:ident $(:: $command_tail:ident)*,)*
      }
      $(@raw {
        $($raw_name:ident ( $($raw_arg:ident : $raw_arg_type:literal),* $(,)? )
          => $raw_head:ident $(:: $raw_tail:ident)*,)*
      })?
      $(@bulk {
        $($route_name:ident ( $($route_arg:ident : $route_arg_type:literal),* $(,)? )
          => $route_head:ident $(:: $route_tail:ident)*,)*
      })?
      $(@jobs {
        $($(#[$job_meta:meta])* $job_name:ident => $job_wire:literal,)*
      })?
    )*
  ) => {
    $(
      pub mod $domain {
        use tauri::Runtime;
        use tauri::ipc::Invoke;

        pub const NAME: &str = $plugin_name;

        $($(
          $(#[$job_meta])*
          #[cfg_attr(feature = "typescript-bindings", xrf_ipc::constant)]
          pub const $job_name: $crate::jobs::JobKind =
            $crate::jobs::JobKind::new(concat!($plugin_name, ".", $job_wire));
        )*)?

        /// The kinds of job this domain runs, each with the name of its constant.
        pub const JOB_KINDS: &[(&str, $crate::jobs::JobKind)] = &[$($((stringify!($job_name), $job_name),)*)?];

        /// Raw commands of this domain as `(wire name, [(argument, TypeScript type)])`.
        ///
        /// Drives generated TypeScript for the commands Specta cannot collect, so an untyped command still has no
        /// hand-written wrapper.
        #[cfg(feature = "typescript-bindings")]
        pub const RAW_COMMANDS: &[(&str, &[(&str, &str)])] = &[
          $($((stringify!($raw_name), &[$((stringify!($raw_arg), $raw_arg_type)),*]),)*)?
        ];

        /// Bulk routes of this domain as `(route name, [(argument, TypeScript type)])`.
        ///
        /// Drives the generated wrappers for what Specta never sees, since a route is not a command, so the one
        /// untyped channel still has no hand-written wrapper.
        #[cfg(feature = "typescript-bindings")]
        pub const BULK_ROUTES: &[(&str, &[(&str, &str)])] = &[
          $($((stringify!($route_name), &[$((stringify!($route_arg), $route_arg_type)),*]),)*)?
        ];

        /// The routes this domain serves over the transport, at `<plugin>/<route>`.
        pub fn routes() -> Vec<$crate::transport::TransportRoute<tauri::AppHandle>> {
          vec![
            $($($crate::transport::TransportRoute::new(
              NAME,
              stringify!($route_name),
              $route_head $(:: $route_tail)*,
            ),)*)?
          ]
        }

        pub fn handler<R: Runtime>() -> impl Fn(Invoke<R>) -> bool + Send + Sync + 'static {
          tauri::generate_handler![
            $($command_head $(:: $command_tail)*,)*
            $($($raw_head $(:: $raw_tail)*,)*)?
          ]
        }

        /// This domain's plugin, named and dispatching its commands, for its crate to add its setup and hooks to.
        pub fn builder<R: Runtime>() -> tauri::plugin::Builder<R> {
          log::info!("Initialize plugin {NAME}");

          tauri::plugin::Builder::new(NAME).invoke_handler($crate::ipc::warn_on_unhandled_command(NAME, handler()))
        }

        #[cfg(feature = "typescript-bindings")]
        pub fn specta_builder<R: Runtime>() -> tauri_specta::Builder<R> {
          // Raw commands are absent from this collection by construction. A command returning
          // `tauri::ipc::Response` cannot be Specta typed at all: tauri blankets `IpcResponse` over
          // every `Serialize`, so a local newtype conflicts, and `#[specta(remote)]` on the foreign
          // type breaks the orphan rule. Declaring one in `@raw { .. }` is therefore the only way to
          // dispatch and permit it, and its wrapper is generated from `RAW_COMMANDS` instead.
          tauri_specta::Builder::new()
            .plugin_name(NAME)
            .error_handling(tauri_specta::ErrorHandlingMode::Throw)
            .commands(tauri_specta::collect_commands![$($command_head $(:: $command_tail)*),*])
            .disable_serde_phases()
            .dangerously_cast_bigints_to_number()
        }

        /// This domain's commands, raw commands and routes, as the frontend's bindings are generated from them.
        #[cfg(feature = "typescript-bindings")]
        pub fn surface<R: Runtime>() -> $crate::ipc::IpcCommandSurface<R> {
          $crate::ipc::IpcCommandSurface::new(NAME, specta_builder(), RAW_COMMANDS, BULK_ROUTES)
        }
      }
    )*

    /// Every route this crate's domains serve, for the transport the application starts.
    pub fn list_routes() -> Vec<$crate::transport::TransportRoute<tauri::AppHandle>> {
      [$($domain::routes(),)*].into_iter().flatten().collect()
    }

    /// Every surface this crate's domains expose to the frontend, as its bindings are generated from them.
    #[cfg(feature = "typescript-bindings")]
    pub fn list_surfaces<R: tauri::Runtime>() -> Vec<$crate::ipc::IpcCommandSurface<R>> {
      vec![$($domain::surface(),)*]
    }

    /// Every kind of job this crate's domains run, each with the name of its constant.
    pub fn list_job_kinds() -> Vec<(&'static str, $crate::jobs::JobKind)> {
      [$($domain::JOB_KINDS,)*].concat()
    }
  };
}
