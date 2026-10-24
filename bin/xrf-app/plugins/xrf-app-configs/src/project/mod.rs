//! The configs projects open: how one is assembled over the shared roots, what it resolved, and what it answers.

mod configs_document;
mod configs_project;
mod configs_project_descriptor;
mod configs_projects;
mod configs_resolved_root;
mod opened_configs_project;

pub use configs_document::ConfigsDocument;
pub use configs_project::ConfigsProject;
pub use configs_project_descriptor::ConfigsProjectDescriptor;
pub use configs_projects::ConfigsProjects;
pub use configs_resolved_root::ConfigsResolvedRoot;
pub use opened_configs_project::OpenedConfigsProject;
