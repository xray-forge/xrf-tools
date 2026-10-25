use crate::manifest::extern_export::ExternExport;
use crate::parse::extern_source_location::ExternSourceLocation;

/// A parsed declaration together with the location used by project projections.
#[derive(Clone, Debug, PartialEq)]
pub struct ParsedExtern {
  pub export: ExternExport,
  pub location: ExternSourceLocation,
  pub name: String,
}
