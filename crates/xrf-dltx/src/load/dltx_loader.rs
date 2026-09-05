use std::collections::BTreeMap;

use xrf_error::{XrfError, XrfResult};
use xrf_ltx::{LtxDocumentSource, LtxItemKind, LtxKeyOperation, LtxSectionOperation};

use crate::discovery::dltx_attachment::DltxAttachment;
use crate::discovery::dltx_discovery::{DLTX_BASE_DEPTH, DltxDiscovery};
use crate::load::dltx_item::DltxItem;
use crate::load::dltx_load_result::DltxLoadResult;
use crate::load::dltx_logical_path::{directory_of, file_name_of};
use crate::resolve::dltx_diagnostic::DltxDiagnostic;

/// Reads a config tree and records every statement it declares, deciding none of them.
///
/// One walk of the root file, its includes, and then its mod files, which is the order the engine loads them in and the
/// reason a mod file may patch a section declared anywhere: nothing is resolved until every file has been read. The
/// deciding is [`crate::resolve::DltxResolver`]'s.
pub struct DltxLoader<'a> {
  source: &'a dyn LtxDocumentSource,
  records: DltxLoadResult,
}

impl<'a> DltxLoader<'a> {
  pub fn new(source: &'a dyn LtxDocumentSource) -> Self {
    Self {
      records: DltxLoadResult::default(),
      source,
    }
  }

  /// Reads `root`, everything it includes, and the mod files attached to it.
  ///
  /// # Errors
  ///
  /// Returns an error when the source cannot answer, or when a statement is one the engine would refuse to start on.
  pub fn load(mut self, root: &str) -> XrfResult<DltxLoadResult> {
    self.load_file(root, DLTX_BASE_DEPTH)?;

    // After the whole base tree, so every section a mod file patches already exists to be patched. Only the root is
    // scanned for them; an included file is never a root.
    for attachment in self.list_attachments(root)? {
      let directory: &str = directory_of(root);
      let path: String = if directory.is_empty() {
        attachment.name
      } else {
        format!("{directory}\\{}", attachment.name)
      };

      self.load_file(&path, attachment.depth)?;
    }

    self.records.create_safe_sections();

    Ok(self.records)
  }

  /// The mod files attached to a root, as names in its own directory.
  fn list_attachments(&self, root: &str) -> XrfResult<Vec<DltxAttachment>> {
    let siblings: Vec<String> = self.source.list_file_names(directory_of(root))?;

    Ok(DltxDiscovery::attachments_of(file_name_of(root), &siblings))
  }

  /// Records one file's statements, recursing into its includes one depth level down.
  fn load_file(&mut self, logical_path: &str, depth: i32) -> XrfResult {
    let Some(document) = self.source.read_document(logical_path)? else {
      return Ok(());
    };

    let filename: String = file_name_of(logical_path).to_lowercase();
    let directory: String = String::from(directory_of(logical_path));

    // Carried across the walk so a field lands in whichever header last opened, exactly as written.
    let mut current: Option<DltxTarget> = None;

    for item in document.get_items() {
      match &item.kind {
        LtxItemKind::Comment { .. } => {}

        LtxItemKind::Include { path, .. } => {
          for included in self.source.resolve_include(&directory, path)? {
            self.load_file(&included, depth + 1)?;
          }
        }

        LtxItemKind::Section {
          name,
          operation,
          parents,
          ..
        } => current = Some(self.open_section(name, *operation, parents, logical_path)?),

        LtxItemKind::Key {
          name, value, operation, ..
        } => match &current {
          Some(target) => self.record_field(target.clone(), name, value.as_deref(), *operation, &filename, depth),
          // Outside any section the engine drops the line without a word.
          None => self.records.diagnostics.push(
            DltxDiagnostic::new("", format!("Field '{name}' sits before any section and is ignored"))
              .with_file(filename.clone())
              .with_engine_behaviour("silently dropped"),
          ),
        },
      }
    }

    Ok(())
  }

  /// Records a section header and answers where the fields under it belong.
  ///
  /// # Errors
  ///
  /// Returns an error when a plain header re-declares a section, which the engine refuses to start on.
  fn open_section(
    &mut self,
    name: &str,
    operation: LtxSectionOperation,
    parents: &[Box<str>],
    path: &str,
  ) -> XrfResult<DltxTarget> {
    // Section names are lowercased at parse time; parent names are not, which is a real trap and preserved as one.
    let section: String = name.to_lowercase();

    match operation {
      LtxSectionOperation::Delete => {
        self.records.deleted_sections.insert(section.clone());

        // Nothing in a `!![section]` body is recorded, so its fields have nowhere to land.
        Ok(DltxTarget::Discarded)
      }

      LtxSectionOperation::Declare => {
        if let Some(previous) = self.records.section_files.get(&section) {
          return Err(XrfError::new_convert_error(format!(
            "Duplicate section '[{section}]' declared in '{previous}' and '{path}' without being marked an override; \
             the engine refuses to start. Use '![{section}]' to patch it"
          )));
        }

        self.records.section_files.insert(section.clone(), String::from(path));
        self.records.base.entry(section.clone()).or_default();
        self.record_parents(&section, parents, false);

        Ok(DltxTarget::Base(section))
      }

      LtxSectionOperation::Override | LtxSectionOperation::SafeOverride => {
        if operation == LtxSectionOperation::SafeOverride {
          self.records.safe_created.insert(section.clone());
        }

        self.records.overrides.entry(section.clone()).or_default();
        self.record_parents(&section, parents, true);

        Ok(DltxTarget::Override(section))
      }
    }
  }

  /// Applies one header's parent tokens to whichever list they belong to.
  ///
  /// Accumulated across every file naming the same section, with a later token cancelling the opposite earlier one.
  /// Removal is the only edit there is: no operator appends a parent, so a bare name is the append.
  fn record_parents(&mut self, section: &str, parents: &[Box<str>], is_override: bool) {
    let declared: &mut Vec<String> = if is_override {
      &mut self.records.override_parents
    } else {
      &mut self.records.base_parents
    }
    .entry(String::from(section))
    .or_default();

    for parent in parents {
      let opposite: String = match parent.strip_prefix('!') {
        Some(name) => String::from(name),
        None => format!("!{parent}"),
      };

      declared.retain(|existing| existing != &opposite);

      if !declared.iter().any(|existing| existing.as_str() == &**parent) {
        declared.push(String::from(&**parent));
      }
    }
  }

  /// Records one field line against the section its header opened.
  fn record_field(
    &mut self,
    target: DltxTarget,
    key: &str,
    value: Option<&str>,
    operation: LtxKeyOperation,
    filename: &str,
    depth: i32,
  ) {
    // The header decides, not the section name: one section may be declared in one file and overridden in another, and
    // a field belongs to whichever of the two it was written under.
    let is_override: bool = match &target {
      DltxTarget::Base(_) => false,
      DltxTarget::Override(_) => true,
      // A field inside a deleted section: the engine records it against nothing, and the section goes anyway.
      DltxTarget::Discarded => return,
    };

    let section: String = match target {
      DltxTarget::Base(section) | DltxTarget::Override(section) => section,
      DltxTarget::Discarded => return,
    };

    let store: &mut BTreeMap<String, Vec<DltxItem>> = match operation {
      // A list operation is routed out of its section whether the header was plain or an override, and keyed by the
      // section name alone.
      LtxKeyOperation::ListAppend | LtxKeyOperation::ListRemove => &mut self.records.list_operations,
      LtxKeyOperation::Set | LtxKeyOperation::Delete if is_override => &mut self.records.overrides,
      LtxKeyOperation::Set | LtxKeyOperation::Delete => &mut self.records.base,
    };

    let items: &mut Vec<DltxItem> = store.entry(section).or_default();

    items.push(DltxItem {
      depth,
      filename: String::from(filename),
      insertion_index: items.len() as u32,
      key: String::from(key),
      operation,
      // A deletion's value is discarded rather than kept, which is what stops it reaching a merged result.
      value: match operation {
        LtxKeyOperation::Delete => None,
        _ => value.map(String::from),
      },
    });
  }
}

/// Where the fields under one header belong.
#[derive(Clone, Debug)]
enum DltxTarget {
  Base(String),
  Override(String),
  /// A `!![section]` header, whose body the engine keeps nowhere.
  Discarded,
}
