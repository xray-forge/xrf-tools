use std::mem;

use xrf_error::{XrfError, XrfResult};

use crate::dialect::{LtxOpenSection, LtxStandardDialect, LtxTextInterner};
use crate::document::{LtxDocument, LtxItem, LtxItemKind};
use crate::ltx::Ltx;

impl LtxStandardDialect {
  /// Lowers one document to a resolved [`Ltx`] under standard LTX rules.
  ///
  /// A rule, not a property of the document: the document accepts a duplicate section, a repeated include and DLTX
  /// operations, and refusing them is this dialect's answer. DLTX refuses a different set, which is why the two live
  /// beside each other as dialects rather than inside the document as one lowering.
  ///
  /// # Errors
  ///
  /// Returns an error when a section is declared twice, an include is named twice, or the file carries a DLTX
  /// operation, which standard mode cannot evaluate.
  pub(crate) fn lower(document: &LtxDocument) -> XrfResult<Ltx> {
    Self::lower_with(document, &mut LtxTextInterner::default())
  }

  /// Lowers one document, sharing its text through an interner the caller owns.
  ///
  /// The door a whole-root resolution uses: one interner spans the root and every file it includes, so a key name
  /// written in fifty of them is stored once.
  ///
  /// # Errors
  ///
  /// The same set [`Self::lower`] refuses.
  pub(crate) fn lower_with(document: &LtxDocument, interner: &mut LtxTextInterner) -> XrfResult<Ltx> {
    let mut ltx: Ltx = Ltx::new();
    let mut open: LtxOpenSection = LtxOpenSection::default();

    ltx.skipped_checks = document.skipped_checks.clone();

    for item in &document.items {
      if item.is_patch_operation() {
        return Err(Self::new_patch_operation_error(item));
      }

      match &item.kind {
        // Carried for formatting and for an editor, and spent by nothing here.
        LtxItemKind::Comment { .. } => {}

        LtxItemKind::Include { path, .. } => {
          if ltx.includes(path) {
            return Err(XrfError::new_ltx_parse_error(
              item.span.get_line(),
              item.span.get_column(),
              format!("Failed to parse include statement in ltx file, including '{path}' more than once"),
            ));
          }

          ltx.include(String::from(&**path));
        }

        LtxItemKind::Section { name, parents, .. } => {
          // Closed first, so a section declared twice is caught by the second header rather than missed because the
          // first one had not landed yet.
          mem::take(&mut open).close_into(&mut ltx);

          if ltx.has_section(name) {
            return Err(XrfError::new_ltx_parse_error(
              item.span.get_line(),
              item.span.get_column(),
              format!("Duplicate sections are not allowed, looks like '{name}' is declared twice"),
            ));
          }

          open = LtxOpenSection::declared(name, parents);
        }

        LtxItemKind::Key { name, value, .. } => open.insert(
          interner.intern(name),
          interner.intern(value.as_deref().unwrap_or_default()),
        ),
      }
    }

    open.close_into(&mut ltx);

    Ok(ltx)
  }

  /// The diagnostic a DLTX statement gets in standard mode.
  ///
  /// Names the flag, because a patch file read without it is a mode mistake rather than a broken config.
  fn new_patch_operation_error(item: &LtxItem) -> XrfError {
    let statement: String = match &item.kind {
      LtxItemKind::Section { name, operation, .. } => {
        format!("section '{}[{name}]'", operation.as_prefix())
      }
      LtxItemKind::Key { name, operation, .. } => {
        format!("field '{}{name}'", operation.as_prefix())
      }
      _ => String::from("statement"),
    };

    XrfError::new_ltx_parse_error(
      item.span.get_line(),
      item.span.get_column(),
      format!("Found DLTX {statement}, which needs the dltx dialect; rerun with --dltx to evaluate patch files"),
    )
  }
}
