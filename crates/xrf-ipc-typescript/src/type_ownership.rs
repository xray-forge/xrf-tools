//! Which module declares which type, and the rules that ownership has to satisfy.

use std::collections::{BTreeMap, BTreeSet};

use specta::Types;
use specta::datatype::NamedDataType;

use crate::mirror_imports::MirrorImports;
use crate::type_modules::TypeModules;
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
  /// Fails when one name resolves to two modules, which would put the same declaration in two files again.
  pub(crate) fn resolve<'a>(
    types: &'a Types,
    modules: &TypeModules,
    imports: &MirrorImports,
  ) -> (Self, BTreeMap<String, Vec<&'a NamedDataType>>) {
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

      if let Some(previous) = ownership.owners.insert(named.name.to_string(), module.clone()) {
        assert_eq!(
          previous, module,
          "`{}` is declared by both `{previous}` and `{module}`",
          named.name
        );
      }

      declared.entry(module).or_default().push(named);
    }

    (ownership, declared)
  }

  /// Records a further name a module declares, so a consumer of it imports from the right module.
  ///
  /// An enum rendered beside the union of a data-free type is not a collected type of its own, yet a command
  /// module naming one still has to import it from wherever that union was written.
  pub(crate) fn declare(&mut self, name: String, module: &str) {
    if let Some(previous) = self.owners.insert(name.clone(), module.to_string()) {
      assert_eq!(
        previous, module,
        "`{name}` is declared by both `{previous}` and `{module}`"
      );
    }
  }

  /// Whether `module` declares a type, or a type's enum, of `name`.
  pub(crate) fn is_declared_by(&self, name: &str, module: &str) -> bool {
    self.owners.get(name).is_some_and(|owner| owner == module)
  }

  /// Names `source` references, excluding the ones `owner` declares itself.
  pub(crate) fn references(&self, source: &str, owner: &str) -> BTreeSet<&str> {
    Self::referenced_types(source, &self.owners, owner)
  }

  /// The modules that have to be imported to satisfy `referenced`.
  pub(crate) fn modules_of(&self, referenced: &BTreeSet<&str>) -> BTreeSet<String> {
    referenced.iter().map(|name| self.owners[*name].clone()).collect()
  }

  pub(crate) fn imports(&self, referenced: &BTreeSet<&str>) -> String {
    self.render_imports(referenced)
  }

  /// The import statements a module carrying `source` needs, where `owner` declares nothing of its own.
  pub(crate) fn imports_for(&self, source: &str) -> String {
    self.imports(&self.references(source, ""))
  }

  /// Fails when generated source names a type declared outside the workspace.
  pub(crate) fn assert_no_foreign_references(&self, source: &str, context: &str) {
    let referenced: BTreeSet<&str> = Self::referenced_types(source, &self.foreign, "");

    assert!(
      referenced.is_empty(),
      "{context} references {referenced:?}, declared outside the workspace. Give them a module before use.",
    );
  }

  /// Names of generated types that `source` references and `owner` does not itself declare.
  pub(crate) fn referenced_types<'a>(
    source: &str,
    owners: &'a BTreeMap<String, String>,
    owner: &str,
  ) -> BTreeSet<&'a str> {
    let blanked: String = without_comments_and_strings(source);
    let mut referenced: BTreeSet<&str> = BTreeSet::new();

    for_each_identifier(&blanked, |_, _, identifier| {
      if let Some((name, module)) = owners.get_key_value(identifier)
        && module != owner
      {
        referenced.insert(name.as_str());
      }
    });

    referenced
  }

  /// Import statements pulling every referenced type from the module that declares it.
  fn render_imports(&self, referenced: &BTreeSet<&str>) -> String {
    let mut grouped: BTreeMap<&str, Vec<&str>> = BTreeMap::new();

    for name in referenced {
      grouped
        .entry(
          self
            .owners
            .get(*name)
            .unwrap_or_else(|| panic!("`{name}` is referenced but has no owning module"))
            .as_str(),
        )
        .or_default()
        .push(name);
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
  /// Crate dependencies are acyclic, so a cycle here means ownership resolved wrongly rather than that the
  /// sources are circular. TypeScript would not report it as an error, only as a partially initialised module.
  pub(crate) fn assert_no_import_cycles(graph: &BTreeMap<String, BTreeSet<String>>) {
    fn walk(
      module: &str,
      graph: &BTreeMap<String, BTreeSet<String>>,
      path: &mut Vec<String>,
      settled: &mut BTreeSet<String>,
    ) {
      if settled.contains(module) {
        return;
      }

      assert!(
        !path.iter().any(|visited| visited == module),
        "Type modules import each other: {} -> {module}",
        path.join(" -> ")
      );

      path.push(module.to_string());

      for imported in graph.get(module).into_iter().flatten() {
        walk(imported, graph, path, settled);
      }

      path.pop();
      settled.insert(module.to_string());
    }

    let mut settled: BTreeSet<String> = BTreeSet::new();

    for module in graph.keys() {
      walk(module, graph, &mut Vec::new(), &mut settled);
    }
  }

  /// The bindings module owning a type, taken from the crate that declares it, as `modules` groups the crates.
  ///
  /// `module_path!()` records the declaring crate as its first segment, so ownership is readable off every
  /// collected type and nothing has to be listed by hand. A hand-written list is what previously let the crate
  /// mirrors fall behind the command modules, duplicating a hundred declarations between them.
  ///
  /// Answers `None` for a type no workspace crate declares, which has no module to live in.
  fn owning_module(named: &NamedDataType, modules: &TypeModules) -> Option<String> {
    modules.module_of(&named.module_path)
  }
}
