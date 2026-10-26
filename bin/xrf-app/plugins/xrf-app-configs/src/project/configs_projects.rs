use xrf_app_core::session::Sessions;

use crate::project::ConfigsProject;

/// The configs projects open, each held by the tool that opened it.
pub type ConfigsProjects = Sessions<ConfigsProject>;
