use std::path::Path;

use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;
use xrf_vfs::{XrayDeclaredRoot, XrayVfs};

use crate::patch::world::ArchivePatchRole;

/// Where a volume set must mount for a patch over it to override anything.
///
/// Every archive of every shipped release declares it: all sixty-eight of Anomaly's volumes, and the default the
/// engine's own `res/fsgame.ltx` writes. An entry point is per-volume while a patch is one volume set declaring one
/// entry point, so a side mounting somewhere else would be compared against the wrong file and overridden at the
/// wrong path.
const GAMEDATA_ENTRY_POINT: &str = "gamedata";

/// Refuse a mounted world holding a volume that would not sit under the gamedata root.
///
/// Read from what each volume declares rather than from where it mounted, because `xrf-vfs` applies no entry point at
/// all: it keys every entry by the name its header authored, so two volumes claiming different roots merge as though
/// they claimed the same one. That is sound for every release anyone ships and silently wrong for anything else,
/// which is exactly why the declaration has to be asked for.
///
/// Every offender is named. The fix is to repack that volume or leave it out, and neither is possible without knowing
/// which one it is.
///
/// # Errors
///
/// Returns an invalid error naming each volume and the root it claims.
pub(crate) fn require_gamedata_entry_point(vfs: &XrayVfs, role: ArchivePatchRole) -> XrfResult<()> {
  let wrong: Vec<String> = vfs
    .get_mounts()
    .iter()
    .flat_map(|mount| mount.get_source().list_declared_roots())
    .filter(|declared| !is_gamedata_root(&declared.root))
    .map(describe)
    .collect();

  if wrong.is_empty() {
    return Ok(());
  }

  Err(XrfError::new_invalid_error(format!(
    "{} {role} volume(s) do not mount at '{GAMEDATA_ENTRY_POINT}': {}. A patch is one volume set declaring one entry \
     point, so entries mounting elsewhere would be compared against the wrong file and overridden at the wrong path.",
    wrong.len(),
    wrong.join(", ")
  )))
}

/// Whether a declared entry point is the gamedata root every shipped release uses.
///
/// An empty root is accepted: a volume carrying no metadata chunk mounts at the gamedata root by the loader's own
/// default, so it agrees with the rule rather than evading it.
fn is_gamedata_root(root: &Path) -> bool {
  let spelled: String = root.to_string_lossy().replace('/', "\\");
  let trimmed: &str = spelled.trim_matches('\\');

  trimmed.is_empty() || trimmed.eq_ignore_ascii_case(GAMEDATA_ENTRY_POINT)
}

fn describe(declared: XrayDeclaredRoot) -> String {
  format!(
    "'{}' declares '{}'",
    format_path(&declared.source),
    declared.root.display()
  )
}
