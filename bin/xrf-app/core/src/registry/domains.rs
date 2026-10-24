// The command domains of this crate's plugins, as a token list. The application's build script includes this file to
// declare the inline plugins and their permissions, and `registry` expands it into the runtime parts, so it holds the
// macro alone. Keep each wire name beside its Rust command path.
// A domain's typed commands may be followed by `@raw { .. }`, then `@bulk { .. }`, in that order; either or both. Each
// entry names its TypeScript arguments, for the generated wrapper Specta cannot write, and its Rust path.
// - `@raw`: commands answering a small `tauri::ipc::Response`, dispatched and permitted like any command. No domain
//   declares one today; the section is kept supported for the next byte answer that fits a command.
// - `@bulk`: routes answering large bytes, served by the loopback transport (`transport/`) so they never touch the
//   window's thread; each route function takes the application and a request deserialized from those arguments.
macro_rules! for_each_core_domain {
  ($consumer:ident) => {
    $consumer! {
      // Bytes of any mounted asset, for every domain: a texture of a model, an entry of an archive, a level's own tree.
      // Reading is generic, so it lives here rather than being reimplemented per domain; what an asset *means* stays with
      // the domain that parses it.
      assets => "assets" {
        commit_text => crate::plugins::assets::commands::commit_text::assets_commit_text,
        detect_engine => crate::plugins::assets::commands::detect_engine::assets_detect_engine,
        list_assets => crate::plugins::assets::commands::list_assets::assets_list_assets,
        probe_root => crate::plugins::assets::commands::probe_root::assets_probe_root,
        subscribe_changes => crate::plugins::assets::commands::subscribe_changes::assets_subscribe_changes,
      }
      @bulk {
        read_asset(roots: "XrayRoots", logicalPath: "string") => crate::plugins::assets::routes::read_asset::assets_read_asset,
      }
      // Running work, whatever domain it belongs to: what is going on, and asking it to stop. A domain of its own
      // because identity, exclusion and cancellation are the same questions for a pack, a verification, or a build.
      jobs => "jobs" {
        attach => crate::plugins::jobs::commands::attach::jobs_attach,
        cancel => crate::plugins::jobs::commands::cancel::jobs_cancel,
        list => crate::plugins::jobs::commands::list::jobs_list,
      }
      // Which tool in which window holds each opening, so whatever a tool, a page or a window leaves is closed.
      sessions => "sessions" {
        bind => crate::plugins::sessions::commands::bind::sessions_bind,
        release_scope => crate::plugins::sessions::commands::release_scope::sessions_release_scope,
        retain_scopes => crate::plugins::sessions::commands::retain_scopes::sessions_retain_scopes,
      }
      // Where the loopback transport listens; the `@bulk` routes are served there rather than over IPC.
      transport => "transport" {
        get_endpoint => crate::plugins::transport::commands::get_endpoint::transport_get_endpoint,
      }
    }
  };
}
