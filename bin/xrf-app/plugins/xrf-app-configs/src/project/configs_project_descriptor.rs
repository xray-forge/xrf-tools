use serde::Serialize;
use xrf_ltx_inspect::LtxInventory;
use xrf_vfs::XrayRoots;

/// What one open of the configs explorer answers with, under the opening's identity.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigsProjectDescriptor {
  /// The trees this project searched, as the backend resolved them, so a reload restores the same open.
  pub roots: XrayRoots,
  /// Scope inside those trees, or nothing for all of them.
  pub prefix: Option<String>,
  /// Whether configs resolve under the Monolith/Anomaly patch dialect.
  pub is_dltx: bool,
  /// Host path the project reports itself at, for a crumb that names something a person recognises.
  pub root: String,
  /// Every config the project holds, and what each one is to it.
  pub inventory: LtxInventory,
  /// Section schemes the project's `*.scheme.ltx` files declare, by name.
  ///
  /// Sent once with the open rather than per document: a tree declares tens of them and every structure read joins
  /// against the same set.
  pub declared_schemes: Vec<String>,
}
