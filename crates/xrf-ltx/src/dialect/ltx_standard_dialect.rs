use xrf_error::{XrfError, XrfResult};

use crate::dialect::{LtxDialect, LtxResolution, LtxTextInterner};
use crate::document::LtxDocument;
use crate::ltx::Ltx;
use crate::source::LtxDocumentSource;

/// Standard LTX: what the vanilla engine and OpenXRay load.
///
/// Includes merge in read order, inheritance resolves eagerly against sections already declared, a duplicate section
/// is refused, and a patch operation is refused because nothing here can evaluate one.
#[derive(Debug, Default)]
pub struct LtxStandardDialect;

impl LtxDialect for LtxStandardDialect {
  fn get_name(&self) -> &'static str {
    "ltx"
  }

  /// Nothing. Standard LTX has no notion of one config patching another, so every config stands on its own.
  fn plan_attachments(&self, _roots: &[String], _source: &dyn LtxDocumentSource) -> XrfResult<Vec<String>> {
    Ok(Vec::new())
  }

  fn resolve(&self, root: &str, source: &dyn LtxDocumentSource) -> XrfResult<LtxResolution> {
    // One interner for the root and every file it includes, so a key name written in fifty of them is stored once.
    // Scoped to the merge: it holds a strong handle to every string it has seen, including text a later file overrides,
    // and nothing after this point creates any.
    let mut ltx: Ltx = Self::merge(root, source, &mut LtxTextInterner::default())?;

    ltx.set_source_paths(root);

    Ok(LtxResolution::new_plain(ltx.into_inherited()?))
  }
}

impl LtxStandardDialect {
  /// Reads one config and merges everything it includes, depth first and in read order.
  fn merge(logical_path: &str, source: &dyn LtxDocumentSource, interner: &mut LtxTextInterner) -> XrfResult<Ltx> {
    let Some(document) = source.read_document(logical_path)? else {
      return Err(XrfError::new_convert_error(format!(
        "Failed to read ltx file, '{logical_path}' is not in scope"
      )));
    };

    Self::merge_document(logical_path, &document, source, interner)
  }

  /// Merges one already-read document's includes, then its own sections on top.
  ///
  /// Includes first and the file's own sections last, which is what lets a config declare root fields beside the
  /// files it pulls in.
  fn merge_document(
    logical_path: &str,
    document: &LtxDocument,
    source: &dyn LtxDocumentSource,
    interner: &mut LtxTextInterner,
  ) -> XrfResult<Ltx> {
    let directory: &str = Ltx::directory_of(logical_path);
    let mut merged: Ltx = Ltx::new();
    let mut included: Vec<String> = Vec::new();

    for statement in document.list_included() {
      if included.iter().any(|held| held == statement) {
        return Err(XrfError::new_convert_error(format!(
          "Failed to parse include statement in ltx file, including '{statement}' more than once"
        )));
      }

      included.push(String::from(statement));

      for path in source.resolve_include(directory, statement)? {
        // An include the world does not hold is nothing to merge, which is the tolerance a not-yet-generated config
        // gets. A wildcard that matches nothing is the same case.
        if let Some(nested) = source.read_document(&path)? {
          merged.merge_sections_from(Self::merge_document(&path, &nested, source, interner)?, &path)?;
        }
      }
    }

    let lowered: Ltx = Self::lower_with(document, interner)?;

    // Only the entry file's opt-out counts, which is what the previous include pass did: an included file's directive
    // is dropped along with the rest of its metadata when its sections are merged.
    merged.skipped_checks = lowered.skipped_checks.clone();

    merged.merge_sections_from(lowered, logical_path)?;

    Ok(merged)
  }
}
