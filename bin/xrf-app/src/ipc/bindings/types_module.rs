//! Writing `core/bindings/types/`, one module per crate that declares an exported type.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use specta::{Format, Types};
use specta_typescript::{Exporter, primitives};

use crate::core::jobs::JobKind;
use crate::ipc::bindings::constants::GENERATED_HEADER;
use crate::ipc::bindings::exporter::{TypeScriptFormat, exporter};
use crate::ipc::bindings::output::write_generated;
use crate::ipc::bindings::ownership::{TypeOwnership, assert_no_import_cycles};

/// Writes one module per declaring crate and answers with the ownership those modules establish.
pub(super) fn export_type_modules(output: &Path, collected: &Types) -> TypeOwnership {
  let types: Types = TypeScriptFormat::default()
    .map_types(collected)
    .expect("Failed to apply the TypeScript format to the collected types")
    .into_owned();
  let exporter: Exporter = exporter().into();
  let (ownership, modules) = TypeOwnership::resolve(&types);

  let mut graph: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
  let mut rendered: BTreeMap<String, String> = BTreeMap::new();

  for (module, named) in &modules {
    let declarations: String = primitives::export(&exporter, &types, named.iter().copied(), "")
      .unwrap_or_else(|error| panic!("Failed to export {module} types: {error}"));

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

  // Runtime names come from the same identities as the serialized JobKind union.
  if let Some(app) = rendered.get_mut("xrf-app") {
    app.push_str(&render_job_kinds());
  }

  for (module, contents) in rendered {
    write_generated(&output.join(format!("{module}.ts")), &contents);
  }

  ownership
}

// todo: Probably render_enum since it can be useful later for us.
fn render_job_kinds() -> String {
  let mut declaration: String = String::from("\n/** Backend job identities. */\nexport enum EJobKind {\n");

  for kind in JobKind::ALL {
    let wire: &str = kind.as_str();
    let member: String = wire.replace(['.', '-'], "_").to_uppercase();
    declaration.push_str(&format!("  {member} = \"{wire}\",\n"));
  }

  declaration.push_str("}\n");
  declaration
}
