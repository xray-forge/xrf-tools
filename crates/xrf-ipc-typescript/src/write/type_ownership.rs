//! Which module declares which type, and the rules that ownership has to satisfy.

use std::collections::{BTreeMap, BTreeSet};

use specta::Types;
use specta::datatype::NamedDataType;
use xrf_error::{XrfError, XrfResult};

use crate::input::{MirrorImports, TypeModules};
use crate::typescript::source::{for_each_identifier, without_comments_and_strings};

/// Where each collected type is declared, split by whether a workspace crate owns it.
pub(crate) struct TypeOwnership {
  /// Type name to the bindings module declaring it.
  owners: BTreeMap<String, String>,
  /// Type name to the foreign module path declaring it, for the ones no bindings module can hold.
  ///
  /// Tauri Specta registers `std::result::Result` whether or not the error handling mode makes use of it. One
  /// that stays unreferenced is simply absent from the output; one that is referenced has nowhere to live.
  foreign: BTreeMap<String, String>,
  /// How an import names the module declaring a type.
  imports: MirrorImports,
}

impl TypeOwnership {
  /// Resolves every collected type to its module as `modules` groups the crates, grouping the declarations each module
  /// has to render; an import of one names it as `imports` addresses it.
  ///
  /// # Errors
  ///
  /// When one name resolves to two modules, which would put the same declaration in two files again.
  pub(crate) fn resolve<'a>(
    types: &'a Types,
    modules: &TypeModules,
    imports: &MirrorImports,
  ) -> XrfResult<(Self, BTreeMap<String, Vec<&'a NamedDataType>>)> {
    let mut ownership: Self = Self {
      owners: BTreeMap::new(),
      foreign: BTreeMap::new(),
      imports: imports.clone(),
    };
    let mut declared: BTreeMap<String, Vec<&NamedDataType>> = BTreeMap::new();

    for named in types.into_sorted_iter() {
      if named.ty.is_none() {
        continue;
      }

      let Some(module) = Self::owning_module(named, modules) else {
        ownership
          .foreign
          .insert(named.name.to_string(), named.module_path.to_string());

        continue;
      };

      ownership.declare(named.name.to_string(), &module)?;
      declared.entry(module).or_default().push(named);
    }

    Ok((ownership, declared))
  }

  /// Records a further name a module declares, so a consumer of it imports from the right module.
  ///
  /// An enum rendered beside the union of a data-free type is not a collected type of its own, yet a command
  /// module naming one still has to import it from wherever that union was written.
  ///
  /// # Errors
  ///
  /// When another module declares the name already.
  pub(crate) fn declare(&mut self, name: String, module: &str) -> XrfResult {
    match self.owners.insert(name.clone(), module.to_string()) {
      Some(previous) if previous != module => Err(XrfError::new_invalid_error(format!(
        "`{name}` is declared by both `{previous}` and `{module}`"
      ))),
      _ => Ok(()),
    }
  }

  /// The module declaring a type, or a type's enum, of `name`, where one does.
  pub(crate) fn owner_of(&self, name: &str) -> Option<&str> {
    self.owners.get(name).map(String::as_str)
  }

  /// Names `source` references, each beside the module declaring it, excluding the ones `owner` declares itself.
  pub(crate) fn references(&self, source: &str, owner: &str) -> BTreeMap<&str, &str> {
    Self::referenced_types(source, &self.owners, owner)
  }

  /// The modules that have to be imported to satisfy `referenced`.
  pub(crate) fn modules_of(referenced: &BTreeMap<&str, &str>) -> BTreeSet<String> {
    referenced.values().map(|module| (*module).to_string()).collect()
  }

  /// The import statements a module carrying `source` needs, where `owner` declares nothing of its own.
  pub(crate) fn imports_for(&self, source: &str) -> String {
    self.imports(&self.references(source, ""))
  }

  /// Fails when generated source names a type declared outside the workspace.
  ///
  /// # Errors
  ///
  /// When `source` references such a type; `context` names where the source was generated for.
  pub(crate) fn assert_no_foreign_references(&self, source: &str, context: &str) -> XrfResult {
    let referenced: BTreeMap<&str, &str> = Self::referenced_types(source, &self.foreign, "");

    if referenced.is_empty() {
      return Ok(());
    }

    Err(XrfError::new_invalid_error(format!(
      "{context} references {:?}, declared outside the workspace. Give them a module before use.",
      referenced.keys().collect::<Vec<_>>()
    )))
  }

  /// Names of generated types that `source` references and `owner` does not itself declare, each beside its module.
  pub(crate) fn referenced_types<'a>(
    source: &str,
    owners: &'a BTreeMap<String, String>,
    owner: &str,
  ) -> BTreeMap<&'a str, &'a str> {
    let blanked: String = without_comments_and_strings(source);
    let mut referenced: BTreeMap<&str, &str> = BTreeMap::new();

    for_each_identifier(&blanked, |_, _, identifier| {
      if let Some((name, module)) = owners.get_key_value(identifier)
        && module != owner
      {
        referenced.insert(name.as_str(), module.as_str());
      }
    });

    referenced
  }

  /// Import statements pulling every referenced type from the module that declares it.
  pub(crate) fn imports(&self, referenced: &BTreeMap<&str, &str>) -> String {
    let mut grouped: BTreeMap<&str, Vec<&str>> = BTreeMap::new();

    for (name, module) in referenced {
      grouped.entry(module).or_default().push(name);
    }

    grouped
      .into_iter()
      .map(|(module, names)| {
        format!(
          "import {{ {} }} from \"{}\";\n",
          names.join(", "),
          self.imports.type_module(module)
        )
      })
      .collect()
  }

  /// Fails when type modules would import each other.
  ///
  /// # Errors
  ///
  /// When `graph` holds a cycle, naming the path that closes it.
  pub(crate) fn assert_no_import_cycles(graph: &BTreeMap<String, BTreeSet<String>>) -> XrfResult {
    let mut settled: BTreeSet<String> = BTreeSet::new();

    for module in graph.keys() {
      Self::walk_imports(module, graph, &mut Vec::new(), &mut settled)?;
    }

    Ok(())
  }

  /// Walks the modules `module` imports depth first, failing on one already on `path`.
  fn walk_imports(
    module: &str,
    graph: &BTreeMap<String, BTreeSet<String>>,
    path: &mut Vec<String>,
    settled: &mut BTreeSet<String>,
  ) -> XrfResult {
    if settled.contains(module) {
      return Ok(());
    }

    if path.iter().any(|visited| visited == module) {
      return Err(XrfError::new_invalid_error(format!(
        "Type modules import each other: {} -> {module}",
        path.join(" -> ")
      )));
    }

    path.push(module.to_string());

    for imported in graph.get(module).into_iter().flatten() {
      Self::walk_imports(imported, graph, path, settled)?;
    }

    path.pop();
    settled.insert(module.to_string());

    Ok(())
  }

  /// The bindings module owning a type, taken from the crate that declares it, as `modules` groups the crates.
  fn owning_module(named: &NamedDataType, modules: &TypeModules) -> Option<String> {
    modules.module_of(&named.module_path)
  }
}
