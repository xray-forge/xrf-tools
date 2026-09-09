use std::collections::HashMap;

use xrf_ltx::{LtxDocument, LtxItem, LtxItemKind};

/// Where a statement sits inside one document.
///
/// The only definition of a section's body a document has: statements carry no section of their own, so a body is
/// everything between one header and the next.
pub(crate) struct LtxDocumentScan {}

impl LtxDocumentScan {
  /// The line one section's header was written on, in the document that declares it.
  ///
  /// A patch dialect lets several files carry the same header, so this answers about the document it was handed and the
  /// caller decides which document that is.
  pub(crate) fn find_section_header_line(document: &LtxDocument, section: &str) -> Option<u32> {
    Self::find_section_header(document, section).map(|(_, item)| item.span.line)
  }

  /// The line one field was written on, inside the body of one section.
  ///
  /// The body is everything between a header and the next one, which is the only definition a document has: statements
  /// carry no section of their own. A field this section inherits rather than writes is not in that range, and answering
  /// `None` for it is the point - a caller anchors that to the header instead.
  pub(crate) fn find_section_key_line(document: &LtxDocument, section: &str, field: &str) -> Option<u32> {
    let (start, _) = Self::find_section_header(document, section)?;

    for item in &document.get_items()[start + 1..] {
      match &item.kind {
        LtxItemKind::Section { .. } => return None,
        LtxItemKind::Key { name, .. } if &**name == field => return Some(item.span.line),
        _ => {}
      }
    }

    None
  }

  /// Every section this document declares, with the parents its header named.
  ///
  /// Built for a whole document at once because the caller asking is filling a resolved index, where one file commonly
  /// declares hundreds of the sections being listed and scanning per section would be quadratic.
  pub(crate) fn list_section_parents(document: &LtxDocument) -> HashMap<String, Vec<String>> {
    let mut parents: HashMap<String, Vec<String>> = HashMap::new();

    for item in document.get_items() {
      if let LtxItemKind::Section {
        name,
        parents: declared,
        ..
      } = &item.kind
      {
        parents.insert(
          String::from(&**name),
          declared.iter().map(|parent| String::from(&**parent)).collect(),
        );
      }
    }

    parents
  }

  /// The first header naming `section`, with its index among the document's statements.
  fn find_section_header<'a>(document: &'a LtxDocument, section: &str) -> Option<(usize, &'a LtxItem)> {
    document
      .get_items()
      .iter()
      .enumerate()
      .find(|(_, item)| match &item.kind {
        LtxItemKind::Section { name, .. } => &**name == section,
        _ => false,
      })
  }
}
