//! Runtime adapter over the command token registry: one module per domain, carrying its dispatch handler,
//! plugin name, and Specta collection.

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
    )*
  ) => {
    $(
      pub(crate) mod $domain {
        use tauri::Runtime;
        use tauri::ipc::Invoke;

        pub(crate) const NAME: &str = $plugin_name;

        /// Raw commands of this domain as `(wire name, [(argument, TypeScript type)])`.
        ///
        /// Drives generated TypeScript for the commands Specta cannot collect, so the one untyped channel
        /// still has no hand-written wrapper.
        #[cfg(feature = "typescript-bindings")]
        pub(crate) const RAW_COMMANDS: &[(&str, &[(&str, &str)])] = &[
          $($((stringify!($raw_name), &[$((stringify!($raw_arg), $raw_arg_type)),*]),)*)?
        ];

        pub(crate) fn handler<R: Runtime>() -> impl Fn(Invoke<R>) -> bool + Send + Sync + 'static {
          tauri::generate_handler![
            $($command_head $(:: $command_tail)*,)*
            $($($raw_head $(:: $raw_tail)*,)*)?
          ]
        }

        #[cfg(feature = "typescript-bindings")]
        pub(crate) fn specta_builder<R: Runtime>() -> tauri_specta::Builder<R> {
          // Raw commands are absent from this collection by construction. A command returning
          // `tauri::ipc::Response` cannot be Specta typed at all: tauri blankets `IpcResponse` over
          // every `Serialize`, so a local newtype conflicts, and `#[specta(remote)]` on the foreign
          // type breaks the orphan rule. Declaring one in `raw { .. }` is therefore the only way to
          // dispatch and permit it, and each one needs a hand written wrapper beside `core/ipc/`.
          tauri_specta::Builder::new()
            .plugin_name(NAME)
            .error_handling(tauri_specta::ErrorHandlingMode::Throw)
            .commands(tauri_specta::collect_commands![$($command_head $(:: $command_tail)*),*])
            .disable_serde_phases()
            .dangerously_cast_bigints_to_number()
        }
      }
    )*
  };
}

for_each_tauri_command_domain!(define_runtime_domains);
