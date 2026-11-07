use std::sync::Arc;

use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{LtxDocument, LtxDocumentSource, LtxItemKind, LtxSectionOperation};

use crate::declaration::{
  LtxDeclaration, LtxDeclarationQuery, LtxDeclarationSearch, LtxDeclarationStatement, LtxUnreadableConfig,
};
use crate::inventory::{LtxInventory, LtxInventoryFile};
use crate::structure::LtxStructureParseError;

/// Searches every config an inventory holds for the statements that write a section or a field.
///
/// Reads documents, never resolutions: the question is where something is written, and resolving would answer a
/// different one - which value survives - while costing a resolution per entry point. The inventory supplies the
/// configs, so this searches exactly what `ltx list` and `ltx verify` see, archived configs included.
pub struct LtxDeclarationReader<'a> {
  inventory: &'a LtxInventory,
  source: &'a dyn LtxDocumentSource,
}

impl<'a> LtxDeclarationReader<'a> {
  /// A reader over one project's inventory and the world its configs came from.
  pub fn new(inventory: &'a LtxInventory, source: &'a dyn LtxDocumentSource) -> Self {
    Self { inventory, source }
  }

  /// Every statement answering `query`, in config order and then in document order.
  ///
  /// # Errors
  ///
  /// Returns an error when a config the inventory lists is no longer in scope, or cannot be read for any reason other
  /// than its own syntax.
  pub fn search(&self, query: &LtxDeclarationQuery) -> XrfResult<LtxDeclarationSearch> {
    let mut declarations: Vec<LtxDeclaration> = Vec::new();
    let mut unreadable: Vec<LtxUnreadableConfig> = Vec::new();

    for file in &self.inventory.files {
      let document: Arc<LtxDocument> = match self.source.read_document(&file.path) {
        Ok(Some(document)) => document,
        Ok(None) => {
          return Err(XrfError::new_read_error(format!(
            "Failed to search ltx declarations, '{}' is not in scope",
            file.path
          )));
        }
        Err(XrfError::LtxParse { line, col, message }) => {
          unreadable.push(LtxUnreadableConfig {
            path: file.path.clone(),
            error: LtxStructureParseError::new(line, col, message),
          });

          continue;
        }
        Err(error) => return Err(error),
      };

      Self::search_document(file, &document, query, &mut declarations);
    }

    Ok(LtxDeclarationSearch {
      query: query.clone(),
      declarations,
      unreadable,
    })
  }

  /// The statements of one document answering `query`, appended in document order.
  ///
  /// A statement carries no section of its own, so the header above it is tracked while walking: a field belongs to
  /// the last header written before it, and to none before the first.
  fn search_document(
    file: &LtxInventoryFile,
    document: &LtxDocument,
    query: &LtxDeclarationQuery,
    declarations: &mut Vec<LtxDeclaration>,
  ) {
    let mut section: Option<&str> = None;
    let mut section_operation: LtxSectionOperation = LtxSectionOperation::Declare;

    for item in document.get_items() {
      match &item.kind {
        LtxItemKind::Section {
          name,
          operation,
          parents,
          ..
        } => {
          section = Some(name);
          section_operation = *operation;

          if query.matches_header(name) {
            declarations.push(LtxDeclaration {
              is_patch: operation.is_patch(),
              is_physical: file.is_physical,
              line: item.span.line,
              path: file.path.clone(),
              section: Some(String::from(&**name)),
              section_operation: String::from(operation.as_prefix()),
              source: file.source.clone(),
              statement: LtxDeclarationStatement::Section {
                parents: parents.iter().map(|parent| String::from(&**parent)).collect(),
              },
            });
          }
        }
        LtxItemKind::Key {
          name, value, operation, ..
        } => {
          if query.matches_field(section, name, value.as_deref()) {
            declarations.push(LtxDeclaration {
              is_patch: operation.is_patch() || section_operation.is_patch(),
              is_physical: file.is_physical,
              line: item.span.line,
              path: file.path.clone(),
              section: section.map(String::from),
              section_operation: String::from(section_operation.as_prefix()),
              source: file.source.clone(),
              statement: LtxDeclarationStatement::Field {
                name: String::from(&**name),
                operation: String::from(operation.as_prefix()),
                value: value.as_deref().map(String::from),
              },
            });
          }
        }
        LtxItemKind::Comment { .. } | LtxItemKind::Include { .. } => {}
      }
    }
  }
}
