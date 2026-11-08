use serde::Serialize;
use xrf_ltx_inspect::LtxDeclarationSearch;

/// What a declaration search over the open project found, as much of it as one answer carries.
///
/// A common field or an empty value matches hundreds of thousands of statements on a modded install, so the answer
/// holds the first `MAXIMUM_DECLARATIONS` in project order and says how many there were; narrowing the query is how a
/// reader sees the rest.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigsDeclarationSearch {
  /// The search, its declarations cut to the first ones in project order.
  pub search: LtxDeclarationSearch,
  /// How many declarations matched before the cut.
  pub total: usize,
}

impl ConfigsDeclarationSearch {
  /// The most declarations one answer carries.
  pub const MAXIMUM_DECLARATIONS: usize = 500;

  /// The answer for `search`, cut to `MAXIMUM_DECLARATIONS`.
  pub fn new(mut search: LtxDeclarationSearch) -> Self {
    let total: usize = search.declarations.len();

    search.declarations.truncate(Self::MAXIMUM_DECLARATIONS);

    Self { search, total }
  }
}
