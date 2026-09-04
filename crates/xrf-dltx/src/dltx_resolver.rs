use std::collections::{BTreeMap, HashMap, HashSet};

use xrf_error::{XrfError, XrfResult};
use xrf_ltx::LtxKeyOperation;

use crate::dltx_diagnostic::DltxDiagnostic;
use crate::dltx_item::DltxItem;
use crate::dltx_provenance::{DltxFieldOrigin, DltxProvenance};
use crate::dltx_stores::DltxStores;

/// One section resolved to its fields, in the order the engine emits them.
type ResolvedSection = BTreeMap<String, DltxItem>;

/// Turns a load pass into resolved sections.
///
/// A depth-first walk with a memo, matching `EvaluateSection`: parents first, then the section's own base merged with
/// its overrides, then list operations, then section deletions. See the compatibility matrix, section 4.
pub struct DltxResolver<'a> {
  stores: &'a DltxStores,
  resolved: HashMap<String, ResolvedSection>,
  /// Sections currently being resolved, which is how a cycle is caught.
  visiting: Vec<String>,
  /// Keys a `!key` removed while merging a base with its overrides, which stops a later list operation reviving them.
  deleted_fields: HashMap<String, HashSet<String>>,
  /// Parents left as declared, after override edits, per section.
  effective_parents: HashMap<String, Vec<String>>,
  diagnostics: Vec<DltxDiagnostic>,
  provenance: DltxProvenance,
}

impl<'a> DltxResolver<'a> {
  pub fn new(stores: &'a DltxStores) -> Self {
    Self {
      deleted_fields: HashMap::new(),
      diagnostics: Vec::new(),
      effective_parents: HashMap::new(),
      provenance: DltxProvenance::default(),
      resolved: HashMap::new(),
      stores,
      visiting: Vec::new(),
    }
  }

  /// Resolves every section the load pass recorded a base for.
  ///
  /// Only base sections are resolved: an override naming a section nothing declares is never reached, which is why
  /// the engine drops it (`Xr_ini.cpp`).
  ///
  /// # Errors
  ///
  /// Returns an error for anything the engine would refuse to start on, which is an inheritance cycle here.
  pub fn resolve_all(mut self) -> XrfResult<DltxResolved> {
    for section in self.stores.base.keys().cloned().collect::<Vec<String>>() {
      self.resolve_section(&section)?;
    }

    self.report_orphan_overrides();

    let mut sections: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();

    // Sorted by section name and by key, which is the order the engine's own container ends up in. Not the authored
    // order: a resolved DLTX document is a lookup table, and reproducing its order is part of matching it.
    for (section, fields) in &self.resolved {
      if self.stores.deleted_sections.contains(section) {
        continue;
      }

      sections.insert(
        section.clone(),
        fields
          .iter()
          .map(|(key, item)| (key.clone(), item.to_resolved_value()))
          .collect(),
      );
    }

    for section in &self.stores.deleted_sections {
      self.provenance.forget_section(section);
    }

    let mut diagnostics: Vec<DltxDiagnostic> = self.stores.diagnostics.clone();

    diagnostics.append(&mut self.diagnostics);

    Ok(DltxResolved {
      diagnostics,
      provenance: self.provenance,
      sections,
    })
  }

  /// Resolves one section, remembering the answer.
  fn resolve_section(&mut self, section: &str) -> XrfResult {
    if self.resolved.contains_key(section) {
      return Ok(());
    }

    if let Some(position) = self.visiting.iter().position(|visited| visited == section) {
      let mut chain: Vec<String> = self.visiting[position..].to_vec();

      chain.push(String::from(section));

      return Err(XrfError::new_convert_error(format!(
        "Inheritance cycle in patched configs: {}; the engine refuses to start",
        chain.join(" -> ")
      )));
    }

    self.visiting.push(String::from(section));

    let parents: Vec<String> = self.resolve_parent_list(section);
    let mut inherited: ResolvedSection = ResolvedSection::new();

    for parent in &parents {
      self.prepare_parent(section, parent)?;
      self.resolve_section(parent)?;

      // Folded left, so a later parent overrides an earlier one.
      if let Some(resolved) = self.resolved.get(parent) {
        for (key, item) in resolved.clone() {
          inherited.insert(key, item);
        }
      }
    }

    let own: ResolvedSection = self.merge_base_with_overrides(section);
    let mut result: ResolvedSection = inherited;

    for (key, item) in own {
      result.insert(key, item);
    }

    // A `!key` in an override removes the inherited value too, which is why deletions apply after the parents fold in.
    if let Some(deleted) = self.deleted_fields.get(section) {
      for key in deleted {
        result.remove(key);
      }
    }

    self.apply_list_operations(section, &mut result);

    for (key, item) in &result {
      self.provenance.record(section, key, DltxFieldOrigin::of(item));
    }

    self.visiting.pop();
    self.resolved.insert(String::from(section), result);

    Ok(())
  }

  /// The parents a section actually inherits, after the override headers have edited the list.
  ///
  /// Override-declared parents are appended, so they outrank every base-declared one, and a `!name` token removes.
  fn resolve_parent_list(&mut self, section: &str) -> Vec<String> {
    if let Some(cached) = self.effective_parents.get(section) {
      return cached.clone();
    }

    let mut tokens: Vec<String> = self.stores.base_parents.get(section).cloned().unwrap_or_default();

    for token in self.stores.override_parents.get(section).cloned().unwrap_or_default() {
      let opposite: String = match token.strip_prefix('!') {
        Some(name) => String::from(name),
        None => format!("!{token}"),
      };

      tokens.retain(|existing| existing != &opposite);

      if !tokens.contains(&token) {
        tokens.push(token);
      }
    }

    let parents: Vec<String> = tokens
      .into_iter()
      .filter(|token| !token.starts_with('!'))
      .collect::<Vec<String>>();

    self.effective_parents.insert(String::from(section), parents.clone());

    parents
  }

  /// Makes sure a named parent can be resolved, reporting it when the engine would have fabricated one.
  fn prepare_parent(&mut self, section: &str, parent: &str) -> XrfResult {
    if self.stores.base.contains_key(parent) {
      return Ok(());
    }

    let reason: &str = if self.stores.overrides.contains_key(parent) {
      "exists only as an override"
    } else {
      "is declared nowhere"
    };

    // Not an error: the engine creates an empty section under that name and carries on, which silently gives the child
    // no inherited fields. A parent name is also not case-folded while a section name is, so `[a]:Base` lands here.
    self.diagnostics.push(
      DltxDiagnostic::new_warning(
        section,
        format!("Inherits '{parent}', which {reason}, so it contributes no fields"),
      )
      .with_engine_behaviour("an empty section is created under that name and appears in the loaded configs"),
    );

    self.resolved.insert(String::from(parent), ResolvedSection::new());

    Ok(())
  }

  /// One section's own fields: its base contents with its overrides applied.
  fn merge_base_with_overrides(&mut self, section: &str) -> ResolvedSection {
    let mut merged: ResolvedSection = ResolvedSection::new();

    for item in Self::rank(self.stores.base.get(section)) {
      match item.operation {
        // A `!key` in a plain section suppresses what the parents offer without recording a deletion, so a later list
        // operation can still revive it (`Xr_ini.cpp:987-990`).
        LtxKeyOperation::Delete => {
          merged.remove(&item.key);
        }
        _ => {
          merged.insert(item.key.clone(), item);
        }
      }
    }

    let Some(overrides) = self.stores.overrides.get(section) else {
      return merged;
    };

    for item in Self::rank(Some(overrides)) {
      match item.operation {
        LtxKeyOperation::Delete => {
          merged.remove(&item.key);

          self
            .deleted_fields
            .entry(String::from(section))
            .or_default()
            .insert(item.key.clone());
        }
        _ => {
          merged.insert(item.key.clone(), item);
        }
      }
    }

    merged
  }

  /// One winner per key: lowest depth, then the latest position within that file.
  ///
  /// The engine sorts by key, then depth ascending, then insertion index descending, and keeps the first of each key
  /// group (`Xr_ini.cpp`). The consequence worth knowing: a root file beats a file it includes for the same
  /// key regardless of which line came first.
  fn rank(items: Option<&Vec<DltxItem>>) -> Vec<DltxItem> {
    let mut winners: BTreeMap<String, DltxItem> = BTreeMap::new();

    for item in items.into_iter().flatten() {
      let is_better: bool = match winners.get(&item.key) {
        None => true,
        Some(existing) => {
          item.depth < existing.depth
            || (item.depth == existing.depth && item.insertion_index > existing.insertion_index)
        }
      };

      if is_better {
        winners.insert(item.key.clone(), item.clone());
      }
    }

    // Back into load order, so a caller reads them as written rather than alphabetically.
    let mut ranked: Vec<DltxItem> = winners.into_values().collect();

    ranked.sort_by_key(|item| (item.depth, item.insertion_index));

    ranked
  }

  /// Applies `>key` and `<key` on top of a fully merged section.
  ///
  /// Last of everything, and cumulative: each operation edits the result of the one before it, in load order
  /// (`Xr_ini.cpp`).
  fn apply_list_operations(&mut self, section: &str, result: &mut ResolvedSection) {
    let Some(operations) = self.stores.list_operations.get(section) else {
      return;
    };

    let mut ordered: Vec<&DltxItem> = operations.iter().collect();

    // By key, then by load position - and deliberately not by depth. A field takes the lowest depth, but a list
    // operation is cumulative, so every one of them applies and they apply in the order they were read: the root
    // file's, then each mod file alphabetically, then top to bottom within a file (`Xr_ini.cpp:1175-1182`).
    ordered.sort_by_key(|item| (item.key.clone(), item.insertion_index));

    for operation in ordered {
      let Some(value) = &operation.value else {
        // `>key =` with nothing after it is skipped outright (`Xr_ini.cpp:1193-1198`).
        continue;
      };

      if self
        .deleted_fields
        .get(section)
        .is_some_and(|deleted| deleted.contains(&operation.key))
      {
        // An override deleted this key, so the operation is consumed and emits nothing.
        continue;
      }

      let mut elements: Vec<String> = result
        .get(&operation.key)
        .map(|item| Self::split_list(&item.to_resolved_value()))
        .unwrap_or_default();

      match operation.operation {
        // No de-duplication: appending something already present yields it twice, as the engine does.
        LtxKeyOperation::ListAppend => elements.extend(Self::split_list(value)),
        LtxKeyOperation::ListRemove => {
          let removed: Vec<String> = Self::split_list(value);

          elements.retain(|element| !removed.contains(element));
        }
        _ => continue,
      }

      if elements.is_empty() {
        // A list edited down to nothing drops its key rather than becoming empty (`Xr_ini.cpp:1272-1275`).
        result.remove(&operation.key);

        continue;
      }

      let mut item: DltxItem = operation.clone();

      item.operation = LtxKeyOperation::Set;
      item.value = Some(elements.join(","));

      result.insert(operation.key.clone(), item);
    }
  }

  /// Splits a comma list the way the engine does: commas only, trimmed, empties dropped.
  fn split_list(value: &str) -> Vec<String> {
    value
      .split(',')
      .map(str::trim)
      .filter(|element| !element.is_empty())
      .map(String::from)
      .collect()
  }

  /// Reports overrides that never found a section to patch.
  fn report_orphan_overrides(&mut self) {
    for section in self.stores.overrides.keys() {
      if self.stores.base.contains_key(section) {
        continue;
      }

      self.diagnostics.push(
        DltxDiagnostic::new_warning(
          section,
          format!("Override '![{section}]' patches a section nothing declares, so it changes nothing"),
        )
        .with_engine_behaviour("silently dropped unless print_dltx_warnings is on, which it is not by default"),
      );
    }
  }
}

/// What a resolution answers with.
#[derive(Debug, Default)]
pub struct DltxResolved {
  /// Sections by lowercased name, each field by key, in the engine's emitted order.
  pub sections: BTreeMap<String, BTreeMap<String, String>>,
  pub provenance: DltxProvenance,
  pub diagnostics: Vec<DltxDiagnostic>,
}

impl DltxResolved {
  /// One field's resolved value.
  pub fn get(&self, section: &str, key: &str) -> Option<&str> {
    self
      .sections
      .get(section)
      .and_then(|fields| fields.get(key))
      .map(String::as_str)
  }

  /// Section names, sorted.
  pub fn list_sections(&self) -> Vec<&str> {
    self.sections.keys().map(String::as_str).collect()
  }

  /// Whether anything would stop the engine starting.
  pub fn has_errors(&self) -> bool {
    self
      .diagnostics
      .iter()
      .any(|diagnostic| diagnostic.severity == crate::dltx_severity::DltxSeverity::Error)
  }
}
