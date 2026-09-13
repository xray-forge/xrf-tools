//! Writing `core/ipc/types/`, one module per crate that declares an exported type.

use std::collections::{BTreeMap, BTreeSet};
use std::iter;
use std::path::Path;

use specta::datatype::NamedDataType;
use specta::{Format, Types};
use specta_typescript::{Exporter, primitives};

use crate::ipc::bindings::constants::GENERATED_HEADER;
use crate::ipc::bindings::enumerations::Enumerations;
use crate::ipc::bindings::exporter::{TypeScriptFormat, exporter};
use crate::ipc::bindings::output::write_generated;
use crate::ipc::bindings::ownership::{TypeOwnership, assert_no_import_cycles};

/// Writes one module per declaring crate, answering with the ownership and the enums those modules establish.
pub(super) fn export_type_modules(output: &Path, collected: &Types) -> (TypeOwnership, Enumerations) {
  let types: Types = TypeScriptFormat::default()
    .map_types(collected)
    .expect("Failed to apply the TypeScript format to the collected types")
    .into_owned();
  let exporter: Exporter = exporter().into();
  let enumerations: Enumerations = Enumerations::resolve(collected, &types);
  let (mut ownership, modules) = TypeOwnership::resolve(&types);

  for (module, named) in &modules {
    for declared in named {
      if let Some(name) = enumerations.enum_name_of(&declared.name) {
        ownership.declare(name, module);
      }
    }
  }

  let mut graph: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
  let mut rendered: BTreeMap<String, String> = BTreeMap::new();

  for (module, named) in &modules {
    let declarations: String = named
      .iter()
      .map(|declared| render_declaration(&exporter, &types, &enumerations, declared, module))
      .collect::<Vec<String>>()
      .join("\n");

    ownership.assert_no_foreign_references(&declarations, module);

    let referenced: BTreeSet<&str> = ownership.references(&declarations, module);

    graph.insert(module.clone(), ownership.modules_of(&referenced));

    // `primitives::export` renders declarations alone, so the file header the high-level exporter would have
    // written has to be prepended here.
    let imports: String = ownership.imports(&referenced);
    let separator: &str = if imports.is_empty() { "" } else { "\n" };

    rendered.insert(
      module.clone(),
      format!("{GENERATED_HEADER}\n{imports}{separator}{declarations}"),
    );
  }

  assert_no_import_cycles(&graph);

  for (module, contents) in rendered {
    write_generated(&output.join(format!("{module}.ts")), &contents);
  }

  (ownership, enumerations)
}

/// One type as the frontend declares it: an enum and its union, or whatever Specta renders it as.
fn render_declaration(
  exporter: &Exporter,
  types: &Types,
  enumerations: &Enumerations,
  declared: &NamedDataType,
  module: &str,
) -> String {
  enumerations.render(declared).unwrap_or_else(|| {
    primitives::export(&exporter, types, iter::once(declared), "")
      .unwrap_or_else(|error| panic!("Failed to export {module} type {}: {error}", declared.name))
  })
}
