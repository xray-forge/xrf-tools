//! The one read that fills a catalog: each config resolved, each section read by its kind's table, then every name a
//! section writes checked against what the engine finds and loads.

pub(crate) mod environment_config;
pub(crate) mod environment_definitions;
pub(crate) mod environment_named;
pub(crate) mod environment_read_pass;
pub(crate) mod environment_read_pass_definitions;
pub(crate) mod environment_read_pass_references;
