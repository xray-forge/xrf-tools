use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use xrf_typescript::swc_common::{SourceMap, sync::Lrc};
use xrf_typescript::swc_ecma_ast::{Decl, ModuleDecl, ModuleItem, Pat, Program};
use xrf_typescript::{parse_typescript_file, render_module_item};

use crate::ipc::bindings::constants::{COMMANDS_DIRECTORY, TYPES_DIRECTORY};
use crate::ipc::bindings::normalization::normalize_module_item;

/// Every exported declaration of a bindings tree, keyed by `<directory>/<file>::<exported name>`.
pub(super) type Surface = BTreeMap<String, String>;

/// How a fresh generation differs from the committed mirrors.
pub(super) struct SurfaceDrift {
  pub removed: Vec<String>,
  pub changed: Vec<String>,
  pub added: Vec<String>,
}

impl SurfaceDrift {
  /// Whether the committed mirrors still describe types the Rust sources produce.
  pub fn is_breaking(&self) -> bool {
    !self.removed.is_empty() || !self.changed.is_empty()
  }

  /// Failure text naming what to regenerate and why, listing additions only as context.
  pub fn describe(&self) -> String {
    let mut report: String = String::from(
      "Committed frontend bindings no longer match the Rust sources.\nRun `cargo make generate-typescript`.\n",
    );

    for (label, entries) in [
      ("Missing from a fresh generation", &self.removed),
      ("Shape changed", &self.changed),
    ] {
      if !entries.is_empty() {
        report.push_str(&format!("\n{label}:\n"));

        for entry in entries {
          report.push_str(&format!("  {entry}\n"));
        }
      }
    }

    if !self.added.is_empty() {
      report.push_str("\nAlso absent from the mirrors, which alone would not fail:\n");

      for entry in &self.added {
        report.push_str(&format!("  {entry}\n"));
      }
    }

    report
  }
}

/// Read every exported declaration of a bindings tree into its canonical form.
pub(super) fn read_surface(root: &Path) -> Surface {
  let mut surface: Surface = Surface::new();

  for directory in [TYPES_DIRECTORY, COMMANDS_DIRECTORY] {
    read_module_directory(&root.join(directory), directory, &mut surface);
  }

  surface
}

/// Classify a committed surface against a freshly generated one.
pub(super) fn compare_surfaces(committed: &Surface, generated: &Surface) -> SurfaceDrift {
  SurfaceDrift {
    removed: committed
      .keys()
      .filter(|key| !generated.contains_key(*key))
      .cloned()
      .collect(),
    changed: committed
      .iter()
      .filter(|(key, declaration)| generated.get(*key).is_some_and(|fresh| fresh != *declaration))
      .map(|(key, _)| key.clone())
      .collect(),
    added: generated
      .keys()
      .filter(|key| !committed.contains_key(*key))
      .cloned()
      .collect(),
  }
}

fn read_module_directory(path: &Path, directory: &str, surface: &mut Surface) {
  let entries = fs::read_dir(path).unwrap_or_else(|error| panic!("Failed to read {}: {error}", path.display()));

  for entry in entries {
    let entry = entry.unwrap_or_else(|error| panic!("Failed to read an entry of {}: {error}", path.display()));
    let file: PathBuf = entry.path();

    if !file.extension().is_some_and(|extension| extension == "ts") {
      continue;
    }

    let name: String = file
      .file_name()
      .unwrap_or_else(|| panic!("Failed to name {}", file.display()))
      .to_string_lossy()
      .into_owned();

    // Joined with `/` rather than the platform separator so a reported key reads the same on every host.
    read_module(&file, &format!("{directory}/{name}"), surface);
  }
}

fn read_module(path: &Path, module: &str, surface: &mut Surface) {
  let source = parse_typescript_file(path)
    .unwrap_or_else(|error| panic!("Failed to parse bindings module {}: {error}", path.display()));

  let Program::Module(parsed) = &source.program else {
    panic!("Bindings module {} did not parse as a module", path.display());
  };

  for item in &parsed.body {
    let ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(export)) = item else {
      continue;
    };

    let declaration: String = canonical_declaration(item, &source.source_map);

    for name in declaration_names(&export.decl) {
      surface.insert(format!("{module}::{name}"), declaration.clone());
    }
  }
}

/// Names an export declaration introduces, which is what a frontend module imports it by.
fn declaration_names(declaration: &Decl) -> Vec<String> {
  match declaration {
    Decl::Class(declared) => vec![declared.ident.sym.to_string()],
    Decl::Fn(declared) => vec![declared.ident.sym.to_string()],
    Decl::TsEnum(declared) => vec![declared.id.sym.to_string()],
    Decl::TsInterface(declared) => vec![declared.id.sym.to_string()],
    Decl::TsTypeAlias(declared) => vec![declared.id.sym.to_string()],
    Decl::Var(declared) => declared
      .decls
      .iter()
      .filter_map(|declarator| match &declarator.name {
        Pat::Ident(binding) => Some(binding.id.sym.to_string()),
        _ => None,
      })
      .collect(),
    // A declaration binding no importable name contributes nothing to compare.
    _ => Vec::new(),
  }
}

/// Canonical text of one declaration, with comments and layout removed.
///
/// Normalizing first is what makes the rendering answer only to shape: the frontend toolchain rewrites
/// syntax as well as whitespace, so two spellings of one type reach here as different trees.
fn canonical_declaration(item: &ModuleItem, source_map: &Lrc<SourceMap>) -> String {
  let mut normalized: ModuleItem = item.clone();

  normalize_module_item(&mut normalized);

  render_module_item(&normalized, source_map)
    .unwrap_or_else(|error| panic!("Failed to render a bindings declaration: {error}"))
}
