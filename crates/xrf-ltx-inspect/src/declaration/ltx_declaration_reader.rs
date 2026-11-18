use std::collections::BTreeSet;
use std::sync::Arc;

use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{LtxDocument, LtxDocumentSource, LtxItemKind, LtxSectionOperation};

use crate::declaration::{
  LtxDeclaration, LtxDeclarationQuery, LtxDeclarationSearch, LtxDeclarationStatement, LtxUnreadableConfig,
};
use crate::inventory::{LtxInventory, LtxInventoryFile, LtxInventoryRole};
use crate::structure::LtxStructureParseError;

/// Searches every config an inventory holds for the statements that write a section or a field.
///
/// Reads documents, never resolutions: resolving would answer which value survives, at a resolution per entry point.
/// It searches exactly the configs `ltx list` and `ltx verify` see, archived ones included.
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
  /// Returns an error when a listed config is no longer in scope, or fails to read for a reason other than its syntax.
  pub fn search(&self, query: &LtxDeclarationQuery) -> XrfResult<LtxDeclarationSearch> {
    let mut declarations: Vec<LtxDeclaration> = Vec::new();
    let mut unreadable: Vec<LtxUnreadableConfig> = Vec::new();

    for file in &self.inventory.files {
      let document: Arc<LtxDocument> = match self.read_document(file) {
        Ok(document) => document,
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

  /// Entry points whose resolution can hold the section `query` names, sorted, or `None` when nothing bounds them.
  ///
  /// A superset read from headers without resolving, so a caller resolves those few rather than every root. A header
  /// counts by naming the section (compared as `query` compares names) or naming it as a parent (compared exactly; the
  /// engine creates an undeclared parent empty), through the entry points above its config. A config that will not
  /// parse counts for none, since neither it nor anything including it resolves.
  ///
  /// `None` when an attachment's plain or `@` header names the section, or any of its headers names it as a parent,
  /// since the attachment plan does not record which root it patches; its `!` or `!!` header only edits an existing
  /// section. A query naming no section is `None` too.
  ///
  /// # Errors
  ///
  /// Returns an error when a listed config is no longer in scope, or fails to read for a reason other than its syntax.
  pub fn list_holding_entry_points(&self, query: &LtxDeclarationQuery) -> XrfResult<Option<Vec<String>>> {
    let Some(section) = query.section.as_deref() else {
      return Ok(None);
    };

    let creating: [LtxSectionOperation; 2] = [LtxSectionOperation::Declare, LtxSectionOperation::SafeOverride];
    let mut holding: BTreeSet<String> = BTreeSet::new();

    for file in &self.inventory.files {
      let document: Arc<LtxDocument> = match self.read_document(file) {
        Ok(document) => document,
        Err(XrfError::LtxParse { .. }) => continue,
        Err(error) => return Err(error),
      };
      let is_attachment: bool = matches!(file.role, LtxInventoryRole::Attachment);

      for item in document.get_items() {
        let LtxItemKind::Section {
          name,
          operation,
          parents,
          ..
        } = &item.kind
        else {
          continue;
        };

        let is_named: bool = query.matches_section_name(name);
        let is_parent: bool = parents.iter().any(|parent| &**parent == section);

        if !is_named && !is_parent {
          continue;
        }

        if !is_attachment {
          holding.extend(self.inventory.list_entry_points_of(&file.path));

          break;
        }

        if is_parent || creating.contains(operation) {
          return Ok(None);
        }
      }
    }

    Ok(Some(holding.into_iter().collect()))
  }

  /// One listed config's document.
  ///
  /// A parse error passes through for the caller to judge. A listed config the source lacks is an error, since both
  /// were read from one project.
  fn read_document(&self, file: &LtxInventoryFile) -> XrfResult<Arc<LtxDocument>> {
    match self.source.read_document(&file.path)? {
      Some(document) => Ok(document),
      None => Err(XrfError::new_read_error(format!(
        "Failed to read ltx declarations, '{}' is not in scope",
        file.path
      ))),
    }
  }

  /// Appends the statements of one document answering `query`, in document order.
  ///
  /// A field belongs to the last header written before it, and to none before the first.
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
