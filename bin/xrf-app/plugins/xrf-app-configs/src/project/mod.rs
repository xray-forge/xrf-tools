//! The configs projects open: how one is assembled over the shared roots, what it resolved, and what it answers.

mod configs_declaration_search;
mod configs_document;
mod configs_project;
mod configs_project_descriptor;
mod configs_projects;
mod configs_resolved_root;
mod opened_configs_project;

pub(crate) use configs_declaration_search::ConfigsDeclarationSearch;
pub(crate) use configs_document::ConfigsDocument;
pub(crate) use configs_project::ConfigsProject;
pub(crate) use configs_project_descriptor::ConfigsProjectDescriptor;
pub use configs_projects::ConfigsProjects;
pub(crate) use configs_resolved_root::ConfigsResolvedRoot;
pub(crate) use opened_configs_project::OpenedConfigsProject;
