use xrf_error::{XrfError, XrfResult};

use crate::document::ltx_document::LtxDocument;
use crate::document::ltx_item::{LtxItem, LtxItemKind};
use crate::document::ltx_span::LtxSpan;
use crate::file::file_section::section::Section;
use crate::file::file_section::section_entry::SectionEntry;
use crate::{Ltx, ROOT_SECTION};

impl LtxDocument {
  /// Lower this document to the resolved-shape [`Ltx`] under standard LTX rules.
  ///
  /// Where the dialect rules live for standard mode. The document itself accepts a duplicate section, a repeated
  /// include, and DLTX operations; refusing them is this pass's job, because DLTX refuses a different set.
  ///
  /// # Errors
  ///
  /// Returns an error when a section is declared twice, an include is named twice, or the file carries a DLTX
  /// operation, which standard mode cannot evaluate.
  pub fn lower(&self) -> XrfResult<Ltx> {
    let mut ltx: Ltx = Ltx::new();
    let mut current_section: String = String::from(ROOT_SECTION);

    ltx.skipped_checks = self.skipped_checks.clone();

    for item in &self.items {
      if item.is_patch_operation() {
        return Err(Self::new_patch_operation_error(item));
      }

      match &item.kind {
        // Carried for formatting and for an editor, and spent by nothing here.
        LtxItemKind::Comment { .. } => {}

        LtxItemKind::Include { path, .. } => {
          if ltx.includes(path) {
            return Err(XrfError::new_ltx_parse_error(
              item.span.line,
              item.span.column,
              format!("Failed to parse include statement in ltx file, including '{path}' more than once"),
            ));
          }

          ltx.include(path.clone());
        }

        LtxItemKind::Section { name, parents, .. } => {
          current_section = name.clone();

          match ltx.entry(current_section.clone()) {
            SectionEntry::Vacant(vacant_entry) => {
              let mut properties: Section = Section::default();

              for parent in parents {
                properties.inherit(parent);
              }

              vacant_entry.insert(properties);
            }
            SectionEntry::Occupied(_) => {
              return Err(XrfError::new_ltx_parse_error(
                item.span.line,
                item.span.column,
                format!("Duplicate sections are not allowed, looks like '{current_section}' is declared twice"),
              ));
            }
          }
        }

        LtxItemKind::Key { name, value, .. } => {
          let value: String = value.clone().unwrap_or_default();

          match ltx.entry(current_section.clone()) {
            SectionEntry::Vacant(vacant_entry) => {
              let mut properties: Section = Section::new();

              properties.insert(name.clone(), value);

              vacant_entry.insert(properties);
            }
            SectionEntry::Occupied(properties) => {
              properties.into_mut().append(name.clone(), value);
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
      line,
      column,
      format!("Found DLTX {statement}, which needs the dltx dialect; rerun with --dltx to evaluate patch files"),
    )
  }
}
