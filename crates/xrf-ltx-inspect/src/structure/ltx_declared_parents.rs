use std::collections::HashMap;

use xrf_error::XrfResult;
use xrf_ltx::LtxDocumentSource;

use crate::structure::LtxDocumentScan;

/// The parents each section's header declared, read back from the configs that declare them.
///
/// Resolving flattens inheritance away - that is what resolving is - so a resolved section carries no parents at all,
/// and the only place they survive is the header in the config that declared it. Reading that back is a lookup rather
/// than a read, because whatever produced the resolution has already parsed and retained every one of those documents.
///
/// Held rather than rebuilt per question: one config commonly declares hundreds of the sections being listed, and a
/// surface that indexes a root and then reads pages out of it would otherwise re-read every declaring config for each
/// page. Keyed by config, not by section, for the same reason.
#[derive(Debug, Default)]
pub(crate) struct LtxDeclaredParents {
  by_config: HashMap<String, HashMap<String, Vec<String>>>,
}

impl LtxDeclaredParents {
  /// The parents one section's header named, in the config that declared it.
  ///
  /// A section whose declaring config is unknown, or which that config no longer declares, answers nothing - the same
  /// answer as a section written with no parents, because neither has any to report.
  ///
  /// # Errors
  ///
  /// Returns an error when the declaring config cannot be read back.
  pub(crate) fn of(&mut self, source: &dyn LtxDocumentSource, origin: &str, section: &str) -> XrfResult<Vec<String>> {
    if !self.by_config.contains_key(origin) {
      let headers: HashMap<String, Vec<String>> = match source.read_document(origin)? {
        Some(document) => LtxDocumentScan::list_section_parents(&document),
        None => HashMap::new(),
      };

      self.by_config.insert(String::from(origin), headers);
    }

    Ok(
      self
        .by_config
        .get(origin)
        .and_then(|headers| headers.get(section))
        .cloned()
        .unwrap_or_default(),
    )
  }
}
