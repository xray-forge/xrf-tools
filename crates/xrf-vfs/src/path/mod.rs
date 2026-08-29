//! X-Ray logical paths: the engine identities every mount, scope and lookup is keyed by.

mod xray_logical_path;
mod xray_path_collision;

pub use xray_logical_path::XrayLogicalPath;
pub(crate) use xray_logical_path::{
  is_component_prefix, join, normalize, normalize_base, normalize_host_relative, normalize_logical, to_host_relative,
};
pub use xray_path_collision::{XrayCollisionSite, XrayPathCollision};
