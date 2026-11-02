//! Host paths for names taken from data, and the forms a host path is shown or resolved in.

mod host_destination;
mod host_file_name;
mod host_link_policy;
mod host_path_forms;
mod host_relative_path;

#[cfg(test)]
mod tests;

pub use host_destination::HostDestination;
pub use host_file_name::HostFileName;
pub use host_link_policy::HostLinkPolicy;
pub use host_path_forms::{format_path, format_path_or, to_absolute_path, to_portable_path_string};
pub use host_relative_path::HostRelativePath;
