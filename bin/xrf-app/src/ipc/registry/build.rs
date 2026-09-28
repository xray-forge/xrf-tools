#[macro_use]
mod domains;

macro_rules! define_inline_plugins {
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
    )*
  ) => {
    fn apply_inline_plugins(attributes: tauri_build::Attributes) -> tauri_build::Attributes {
      attributes$(
        .plugin(
          $plugin_name,
          tauri_build::InlinedPlugin::new()
            // Raw commands are permitted like any other: the ACL governs dispatch, and only the Specta collection
            // excludes them. Bulk routes are not commands: the transport serves them, behind its own token, so the
            // ACL never sees them.
            .commands(&[
              $(stringify!($command_name),)*
              $($(stringify!($raw_name),)*)?
            ])
            .default_permission(tauri_build::DefaultPermissionRule::AllowAllCommands),
        )
      )*
    }
  };
}

for_each_tauri_command_domain!(define_inline_plugins);

pub fn configure(attributes: tauri_build::Attributes) -> tauri_build::Attributes {
  apply_inline_plugins(attributes)
}
