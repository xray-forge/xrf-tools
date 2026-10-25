use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::extern_export::ExternExport;

/// The public, stable manifest of names exported to X-Ray configuration data.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct ExternManifest {
  pub exports: BTreeMap<String, ExternExport>,
}
