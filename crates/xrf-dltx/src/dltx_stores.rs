use std::collections::{BTreeMap, BTreeSet};
use xrf_error::XrfResult;
use xrf_ltx::{LtxDocumentSource, LtxItemKind, LtxKeyOperation, LtxSectionOperation};

use crate::dltx_attachment::DltxAttachment;
use crate::dltx_diagnostic::DltxDiagnostic;
use crate::dltx_discovery::{DLTX_BASE_DEPTH, DltxDiscovery};
use crate::dltx_item::DltxItem;

/// Everything one load pass records, before any of it is resolved.
///
/// The engine fills the same set of maps in one walk of the root file, its includes, and its mod files, then resolves
/// from them. Keeping the two apart is what lets a later section be patched by an earlier file: nothing is decided
/// until every file has been read. See the compatibility matrix, section 4.
#[derive(Debug, Default)]
pub struct DltxStores {
  /// Plain sections, by lowercased name.
  pub base: BTreeMap<String, Vec<DltxItem>>,
  /// `![section]` and `@[section]` contents, by lowercased name.
  pub overrides: BTreeMap<String, Vec<DltxItem>>,
  /// `>key` and `<key` operations, by lowercased section name, in the order read.
  pub list_operations: BTreeMap<String, Vec<DltxItem>>,
  /// `!![section]` targets, applied after everything else resolves.
  pub deleted_sections: BTreeSet<String>,
  /// Parent tokens declared by plain headers. A token may carry a leading `!`, meaning removal.
  pub base_parents: BTreeMap<String, Vec<String>>,
  /// Parent tokens declared by override headers.
  pub override_parents: BTreeMap<String, Vec<String>>,
  /// `@[section]` names, which get an empty base section when nothing else declares one.
  pub safe_created: BTreeSet<String>,
  /// Which file declared each plain section, for the duplicate diagnostic.
  pub section_files: BTreeMap<String, String>,
  /// What the load pass found worth reporting.
  pub diagnostics: Vec<DltxDiagnostic>,
}

impl DltxStores {
  /// Reads a root config, its includes, and its mod files, recording everything they declare.
  ///
  /// # Errors
  ///
  /// Returns an error when the source cannot answer, or when a statement is one the engine would refuse to start on.
  pub fn load(source: &dyn LtxDocumentSource, root: &str) -> XrfResult<Self> {
    let mut stores: Self = Self::default();

    stores.load_file(source, root, DLTX_BASE_DEPTH)?;

    // Mod files come after the whole base tree, so every section they patch already exists to be patched. Only the
    // root file is scanned for them; an included file is never a root (`Xr_ini.cpp:537-544,350-359`).
    for attachment in Self::attachments_of(source, root)? {
      let path: String = match Self::directory_of(root) {
        Some(directory) => format!("{directory}\\{}", attachment.name),
        None => attachment.name.clone(),
      };

      stores.load_file(source, &path, attachment.depth)?;
    }

    stores.create_safe_sections();

    Ok(stores)
  }

  /// The mod files attached to a root, as file names in its own directory.
  fn attachments_of(source: &dyn LtxDocumentSource, root: &str) -> XrfResult<Vec<DltxAttachment>> {
    let directory: &str = Self::directory_of(root).unwrap_or("");
    let siblings: Vec<String> = source.list_file_names(directory)?;

    Ok(DltxDiscovery::attachments_of(Self::file_name_of(root), &siblings))
  }

  /// Records one file's statements, recursing into its includes one depth level down.
  fn load_file(&mut self, source: &dyn LtxDocumentSource, logical_path: &str, depth: i32) -> XrfResult {
    let Some(document) = source.read_document(logical_path)? else {
      return Ok(());
    };

    let filename: String = Self::file_name_of(logical_path).to_lowercase();
    let directory: &str = Self::directory_of(logical_path).unwrap_or("");

    // Kept as the walk proceeds so a field lands in whichever section header last opened, exactly as written.
    let mut current: Option<DltxTarget> = None;

    for item in document.get_items() {
      match &item.kind {
        LtxItemKind::Comment { .. } => {}

        LtxItemKind::Include { path, .. } => {
          for included in source.resolve_include(directory, path)? {
            self.load_file(source, &included, depth + 1)?;
          }
        }

        LtxItemKind::Section {
          name,
          operation,
          parents,
          ..
        } => {
          current = Some(self.open_section(name, *operation, parents, &filename)?);
        }

        LtxItemKind::Key {
          name, value, operation, ..
        } => {
          let Some(target) = &current else {
            // Outside any section the engine drops the line without a word (`Xr_ini.cpp:876-879`).
            self.diagnostics.push(
              DltxDiagnostic::new_warning("", format!("Field '{name}' sits before any section and is ignored"))
                .with_file(filename.clone())
                .with_engine_behaviour("silently dropped"),
            );

            continue;
          };

          self.record_field(target.clone(), name, value.clone(), *operation, &filename, depth)?;
        }
      }
    }

    Ok(())
  }

  /// Records a section header and answers where its fields belong.
  fn open_section(
    &mut self,
    name: &str,
    operation: LtxSectionOperation,
    parents: &[String],
    filename: &str,
  ) -> XrfResult<DltxTarget> {
    // Section names are lowercased at parse time; parent names are not, which is a real trap and preserved as one.
    let section: String = name.to_lowercase();

    match operation {
      LtxSectionOperation::Delete => {
        self.deleted_sections.insert(section.clone());

        // Nothing in a `!![section]` body is recorded, so its fields have nowhere to land.
        Ok(DltxTarget::Discarded(section))
      }
      LtxSectionOperation::Declare => {
        if let Some(previous) = self.section_files.get(&section) {
          return Err(xrf_error::XrfError::new_convert_error(format!(
            "Duplicate section '[{section}]' declared in '{previous}' and '{filename}' without being marked an \
             override; the engine refuses to start. Use '![{section}]' to patch it"
          )));
        }

        self.section_files.insert(section.clone(), String::from(filename));
        self.base.entry(section.clone()).or_default();
        self.record_parents(&section, parents, false);

        Ok(DltxTarget::Base(section))
      }
      LtxSectionOperation::Override | LtxSectionOperation::SafeOverride => {
        if operation == LtxSectionOperation::SafeOverride {
          self.safe_created.insert(section.clone());
        }

        self.overrides.entry(section.clone()).or_default();
        self.record_parents(&section, parents, true);

        Ok(DltxTarget::Override(section))
      }
    }
  }

  /// Applies one header's parent tokens to whichever list they belong to.
  ///
  /// Accumulated across every file that names the same section, with a later token cancelling the opposite earlier one.
  /// Only removal exists as an edit: there is no operator that appends a parent, so a bare name is the append
  /// (`Xr_ini.cpp:300-328`).
  fn record_parents(&mut self, section: &str, parents: &[String], is_override: bool) {
    let target: &mut BTreeMap<String, Vec<String>> = if is_override {
      &mut self.override_parents
    } else {
      &mut self.base_parents
    };

    let declared: &mut Vec<String> = target.entry(String::from(section)).or_default();

    for parent in parents {
      let opposite: String = match parent.strip_prefix('!') {
        Some(name) => String::from(name),
        None => format!("!{parent}"),
      };

      declared.retain(|existing| existing != &opposite);

      if !declared.contains(parent) {
        declared.push(parent.clone());
      }
    }
  }

  /// Records one field line into its section's store.
  fn record_field(
    &mut self,
    target: DltxTarget,
    key: &str,
    value: Option<String>,
    operation: LtxKeyOperation,
    filename: &str,
    depth: i32,
  ) -> XrfResult {
    let section: String = match &target {
      DltxTarget::Base(section) | DltxTarget::Override(section) => section.clone(),
      // A field inside a deleted section: the engine records it against nothing, and the section goes anyway.
      DltxTarget::Discarded(_) => return Ok(()),
    };

    let store: &mut BTreeMap<String, Vec<DltxItem>> = match operation {
      // A list operation is routed out of its section whether the header was plain or an override, keyed by the
      // section name alone (`Xr_ini.cpp:241-251`).
      LtxKeyOperation::ListAppend | LtxKeyOperation::ListRemove => &mut self.list_operations,
      LtxKeyOperation::Set | LtxKeyOperation::Delete => match &target {
        DltxTarget::Base(_) => &mut self.base,
        DltxTarget::Override(_) => &mut self.overrides,
        DltxTarget::Discarded(_) => return Ok(()),
      },
    };

    let items: &mut Vec<DltxItem> = store.entry(section).or_default();
    let insertion_index: u32 = items.len() as u32;

    items.push(DltxItem {
      depth,
      filename: String::from(filename),
      insertion_index,
      key: String::from(key),
      operation,
      // A deletion's value is discarded rather than kept, which is what stops it reaching a merged result.
      value: match operation {
        LtxKeyOperation::Delete => None,
        _ => value,
      },
    });

    Ok(())
  }

  /// Gives every `@[section]` an empty base section when nothing declared one.
  ///
  /// What separates a safe override from a plain one: the section is created rather than the override being dropped,
  /// and it bypasses the duplicate refusal (`Xr_ini.cpp:892-905`).
  fn create_safe_sections(&mut self) {
    for section in &self.safe_created {
      self.base.entry(section.clone()).or_default();
    }
  }

  /// The last `\`-separated segment of a logical path.
  fn file_name_of(logical_path: &str) -> &str {
    logical_path.rsplit_once('\\').map_or(logical_path, |(_, name)| name)
  }

  /// Everything before the last `\`, or `None` for a path with no directory.
  fn directory_of(logical_path: &str) -> Option<&str> {
    logical_path.rsplit_once('\\').map(|(directory, _)| directory)
  }
}

/// Where the fields under a header belong.
#[derive(Clone, Debug)]
enum DltxTarget {
  Base(String),
  Override(String),
  /// A `!![section]` header, whose body the engine keeps nowhere.
  Discarded(String),
}
