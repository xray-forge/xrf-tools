//! Reading everything at once: the catalog, and the one read that fills it.

pub(crate) mod environment_catalog;
pub(crate) mod environment_config;
pub(crate) mod environment_definitions;
pub(crate) mod environment_named;
pub(crate) mod environment_read_options;
pub(crate) mod environment_read_pass;
pub(crate) mod environment_read_pass_definitions;
pub(crate) mod environment_read_pass_references;
pub(crate) mod environment_reader;

pub use crate::catalog::environment_catalog::EnvironmentCatalog;
pub use crate::catalog::environment_read_options::EnvironmentReadOptions;
pub use crate::catalog::environment_reader::EnvironmentReader;
