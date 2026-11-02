// This crate's command domain, as a token list: the application's build script includes this file to declare the inline
// plugin and its permissions, and `ipc` expands it into the runtime parts, so it holds the macro alone. Keep each
// wire name beside its Rust command path; `xrf_app_core::ipc` describes the sections.
macro_rules! for_each_gamedata_domain {
  ($consumer:ident) => {
    $consumer! {
      gamedata => "gamedata" {
        verify_project => crate::commands::verify_project::gamedata_verify_project,
      }
      @jobs {
        /// Verifying a gamedata project.
        GAMEDATA_JOB_VERIFY => "verify",
      }
    }
  };
}
