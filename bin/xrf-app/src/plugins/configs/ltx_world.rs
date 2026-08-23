use xrf_error::XrfResult;
use xrf_ltx::{LtxProject, LtxProjectOptions};
use xrf_vfs::{XrayLookupScope, XrayVfs, XrayWorldSpec};

/// Open an LTX project over a world.
///
/// The world says which trees are searched and how each is read, so an installation is verified as
/// readily as a loose tree — its configs come out of `db\configs` and a reader reaching for the
/// filesystem would report them absent.
///
/// `prefix` is the layout half: the logical subtree holding the configs. Absent means the whole world,
/// which is what a caller pointing straight at a configs directory wants.
///
/// # Errors
///
/// Returns an error when the world cannot be mounted, the prefix is not a logical path, or the project
/// cannot be assembled.
pub fn open_ltx_project(
  world: &XrayWorldSpec,
  prefix: Option<&str>,
  options: LtxProjectOptions,
) -> XrfResult<LtxProject> {
  let vfs: XrayVfs = world.open()?;
  let scope: XrayLookupScope = match prefix {
    Some(prefix) => XrayLookupScope::all().with_prefix(prefix)?,
    None => XrayLookupScope::all(),
  };

  LtxProject::open_at_scope_opt(world.describe(), vfs, scope, options)
}
