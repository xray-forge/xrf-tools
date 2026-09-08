use std::path::PathBuf;

/// One place a source's own metadata says its entries mount, and the container that declared it.
///
/// This crate applies none of them. An archive entry is keyed by the name its header authored, whatever `[header]
/// entry_point` beside it claims, so two volumes declaring different roots merge as though they declared the same
/// one. That is sound for every shipped release, which declares the gamedata root and nothing else, and wrong for
/// anything that does not — so a consumer whose answer depends on where entries really land asks for this and
/// refuses what it cannot honour, rather than being silently wrong.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct XrayDeclaredRoot {
  /// The volume file whose metadata declared it.
  pub source: PathBuf,
  /// The logical root, with its `$alias$` stripped, as `gamedata/` for an ordinary archive.
  pub root: PathBuf,
}
