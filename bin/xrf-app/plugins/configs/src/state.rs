use xrf_app_core::session::Sessions;

use crate::configs_project::ConfigsProject;

/// The configs projects open, each held by the tool that opened it.
pub type ConfigsState = Sessions<ConfigsProject>;
