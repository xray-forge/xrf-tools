use xrf_error::XrfResult;
use xrf_ltx::{LtxProject, LtxProjectOptions};
use xrf_vfs::{XrayCachePolicy, XrayLookupScope, XrayRoots, XrayVfs};

/// Open an LTX project over roots.
///
/// The roots says which trees are searched and how each is read, so an installation is verified as
/// readily as a loose tree — its configs come out of `db\configs` and a reader reaching for the
/// filesystem would report them absent.
///
/// `prefix` is the layout half: the logical subtree holding the configs. Absent means the whole roots,
/// which is what a caller pointing straight at a configs directory wants.
///
/// # Errors
///
/// Returns an error when the roots cannot be mounted, the prefix is not a logical path, or the project
/// cannot be assembled.
pub fn open_ltx_project(roots: &XrayRoots, prefix: Option<&str>, options: LtxProjectOptions) -> XrfResult<LtxProject> {
  let vfs: XrayVfs = roots.open()?.with_cache_policy(XrayCachePolicy::configs());
  let scope: XrayLookupScope = match prefix {
    Some(prefix) => XrayLookupScope::all().with_prefix(prefix)?,
    None => XrayLookupScope::all(),
  };

  LtxProject::open_at_scope_opt(roots.describe(), vfs, scope, options)
}
