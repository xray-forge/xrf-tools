use xrf_error::{XrfError, XrfResult};

use crate::dialect::LtxStandardDialect;
use crate::document::{LtxDocument, LtxItem, LtxItemKind, LtxSpan};
use crate::ltx::{Ltx, Section, SectionEntry};
use crate::syntax::ROOT_SECTION;

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
    let mut ltx: Ltx = Ltx::new();
    let mut current_section: String = String::from(ROOT_SECTION);

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
          current_section = String::from(&**name);

          match ltx.entry(current_section.clone()) {
            SectionEntry::Vacant(vacant_entry) => {
              let mut properties: Section = Section::default();

              for parent in parents {
                properties.inherit(&**parent);
              }

              vacant_entry.insert(properties);
            }
            SectionEntry::Occupied(_) => {
              return Err(XrfError::new_ltx_parse_error(
                item.span.get_line(),
                item.span.get_column(),
                format!("Duplicate sections are not allowed, looks like '{current_section}' is declared twice"),
              ));
            }
          }
        }

        LtxItemKind::Key { name, value, .. } => {
          let value: &str = value.as_deref().unwrap_or_default();

          match ltx.entry(current_section.clone()) {
            SectionEntry::Vacant(vacant_entry) => {
              let mut properties: Section = Section::new();

              properties.insert(&**name, value);

              vacant_entry.insert(properties);
            }
            SectionEntry::Occupied(properties) => {
              properties.into_mut().insert(&**name, value);
            }
          }
        }
      }
    }

    Ok(ltx)
  }

  /// The diagnostic a DLTX statement gets in standard mode.
  ///
  /// Names the flag, because a patch file read without it is a mode mistake rather than a broken config.
  fn new_patch_operation_error(item: &LtxItem) -> XrfError {
    let LtxSpan { column, line } = item.span;

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
      line as usize,
      column as usize,
      format!("Found DLTX {statement}, which needs the dltx dialect; rerun with --dltx to evaluate patch files"),
    )
  }
}
