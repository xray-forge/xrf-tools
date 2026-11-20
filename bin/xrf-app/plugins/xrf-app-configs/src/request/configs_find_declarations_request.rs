use serde::{Deserialize, Serialize};
use xrf_app_core::session::SessionId;
use xrf_ltx_inspect::LtxDeclarationQuery;

/// A search for where the open project writes a section, a field or a value; an empty one is taken as not given.
///
/// It carries no name comparison: the backend takes that from the project's dialect.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub(crate) struct ConfigsFindDeclarationsRequest {
  /// The open project searched; a closed one is refused.
  pub session_id: SessionId,
  /// Section name without brackets or operation prefix.
  pub section: Option<String>,
  /// Field name without operation prefix.
  pub field: Option<String>,
  /// Text a written value must contain.
  pub value: Option<String>,
}

impl ConfigsFindDeclarationsRequest {
  /// The query asked, an empty criterion taken as not given, or none when nothing is left to look for.
  pub(crate) fn get_query(&self) -> Option<LtxDeclarationQuery> {
    let given = |criterion: &Option<String>| criterion.clone().filter(|it| !it.is_empty());
    let query: LtxDeclarationQuery = LtxDeclarationQuery {
      section: given(&self.section),
      field: given(&self.field),
      value: given(&self.value),
      ..LtxDeclarationQuery::default()
    };

    (query.section.is_some() || query.field.is_some() || query.value.is_some()).then_some(query)
  }
}
