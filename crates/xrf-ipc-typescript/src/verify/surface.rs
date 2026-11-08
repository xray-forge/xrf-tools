use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use xrf_error::{XrfError, XrfResult};
use xrf_typescript::swc_common::SourceMap;
use xrf_typescript::swc_common::sync::Lrc;
use xrf_typescript::swc_ecma_ast::{Decl, ModuleDecl, ModuleItem, Pat, Program};
use xrf_typescript::{TypeScriptItemRenderer, TypeScriptSource};
use xrf_utils::format_path;

use crate::typescript::normalization::normalize_module_item;
use crate::verify::SurfaceDrift;
use crate::write::GeneratedOutput;

/// Every exported declaration of a bindings tree, keyed by `<directory>/<file>::<exported name>`.
pub(crate) struct Surface {
  declarations: BTreeMap<String, String>,
}

impl Surface {
  /// Reads every exported declaration of a bindings tree into its canonical form.
  ///
  /// # Errors
  ///
  /// When a directory cannot be read, or a module in it does not parse as a TypeScript module.
  pub(crate) fn read(root: &Path) -> XrfResult<Self> {
    let mut surface: Self = Self {
      declarations: BTreeMap::new(),
    };

    for directory in [GeneratedOutput::TYPES_DIRECTORY, GeneratedOutput::COMMANDS_DIRECTORY] {
      surface.read_directory(&root.join(directory), directory)?;
    }

    Ok(surface)
  }

  /// How this surface differs from a freshly generated one.
  pub(crate) fn compare_to(&self, generated: &Self) -> SurfaceDrift {
    SurfaceDrift {
      removed: self
        .declarations
        .keys()
        .filter(|key| !generated.declarations.contains_key(*key))
        .cloned()
        .collect(),
      changed: self
        .declarations
        .iter()
        .filter(|(key, declaration)| {
          generated
            .declarations
            .get(*key)
            .is_some_and(|fresh| fresh != *declaration)
        })
        .map(|(key, _)| key.clone())
        .collect(),
      added: generated
        .declarations
        .keys()
        .filter(|key| !self.declarations.contains_key(*key))
        .cloned()
        .collect(),
    }
  }

  /// Reads one generated directory, keying every declaration by the module it is exported from.
  fn read_directory(&mut self, path: &Path, directory: &str) -> XrfResult {
    let entries = fs::read_dir(path).map_err(|error| GeneratedOutput::describe_io("read", path, &error))?;

    for entry in entries {
      let entry = entry.map_err(|error| GeneratedOutput::describe_io("read an entry of", path, &error))?;
      let file: PathBuf = entry.path();

      if !file.extension().is_some_and(|extension| extension == "ts") {
        continue;
      }

      // Joined with `/` rather than the platform separator so a reported key reads the same on every host.
      self.read_module(&file, &format!("{directory}/{}", entry.file_name().to_string_lossy()))?;
    }

    Ok(())
  }

  /// Reads one module, canonicalizing each exported declaration.
  fn read_module(&mut self, path: &Path, module: &str) -> XrfResult {
    let source: TypeScriptSource = TypeScriptSource::parse_file(path).map_err(|error| {
      XrfError::new_parsing_error(format!(
        "Failed to parse bindings module {}: {error}",
        format_path(path)
      ))
    })?;

    let Program::Module(parsed) = &source.program else {
      return Err(XrfError::new_parsing_error(format!(
        "Bindings module {} did not parse as a module",
        format_path(path)
      )));
    };

    for item in &parsed.body {
      let ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(export)) = item else {
        continue;
      };

      let declaration: String = Self::canonicalize(item, &source.source_map)?;

      for name in Self::list_names(&export.decl) {
        self
          .declarations
          .insert(format!("{module}::{name}"), declaration.clone());
      }
    }

    Ok(())
  }

  /// Names an export declaration introduces, which is what a frontend module imports it by.
  fn list_names(declaration: &Decl) -> Vec<String> {
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
  fn canonicalize(item: &ModuleItem, source_map: &Lrc<SourceMap>) -> XrfResult<String> {
    let mut normalized: ModuleItem = item.clone();

    normalize_module_item(&mut normalized);

    TypeScriptItemRenderer::render(&normalized, source_map)
      .map_err(|error| XrfError::new_serialization_error(format!("Failed to render a bindings declaration: {error}")))
  }
}

#[cfg(test)]
mod tests {
  use std::fs;
  use std::path::PathBuf;

  use xrf_error::XrfResult;
  use xrf_test_utils::GeneratedResources;

  use crate::verify::{Surface, SurfaceDrift};
  use crate::write::GeneratedOutput;

  /// The surface of a bindings tree whose one type module holds `source`.
  fn read(name: &str, source: &str) -> XrfResult<Surface> {
    let root: PathBuf = GeneratedResources::path("bindings-surface").join(name);

    for directory in [GeneratedOutput::TYPES_DIRECTORY, GeneratedOutput::COMMANDS_DIRECTORY] {
      fs::create_dir_all(root.join(directory))?;
    }

    fs::write(
      root.join(GeneratedOutput::TYPES_DIRECTORY).join("xrf-fixture.ts"),
      source,
    )?;

    Surface::read(&root)
  }

  #[test]
  fn reads_an_exported_constant_and_breaks_on_its_value() -> XrfResult {
    let committed: Surface = read("committed", "export const LIMIT: number = 4;\n")?;
    let documented: Surface = read("documented", "/** The most. */\nexport const LIMIT: number = 4;\n")?;
    let changed: SurfaceDrift = committed.compare_to(&read("changed", "export const LIMIT: number = 8;\n")?);

    assert!(!committed.compare_to(&documented).is_breaking());
    assert_eq!(changed.changed, ["types/xrf-fixture.ts::LIMIT"]);
    assert!(changed.is_breaking());

    Ok(())
  }

  // The generator writes an array constant's type as `Entry[]`, which the formatter rewrites as `Array<Entry>`.
  #[test]
  fn reads_an_array_constant_alike_in_both_spellings_of_its_type() -> XrfResult {
    let committed: Surface = read(
      "array-committed",
      "export const KINDS: Array<Entry> = [\n  {\n    kind: \"props\",\n  },\n];\n",
    )?;
    let fresh: Surface = read("array-fresh", "export const KINDS: Entry[] = [{ kind: \"props\" }];\n")?;
    let retyped: SurfaceDrift = committed.compare_to(&read(
      "array-retyped",
      "export const KINDS: Array<Other> = [{ kind: \"props\" }];\n",
    )?);

    assert!(!committed.compare_to(&fresh).is_breaking());
    assert_eq!(retyped.changed, ["types/xrf-fixture.ts::KINDS"]);

    Ok(())
  }

  #[test]
  fn refuses_a_module_that_does_not_parse_rather_than_panicking() {
    let error: String = read("unparsed", "export const = ;\n")
      .err()
      .map(|error| error.to_string())
      .unwrap_or_default();

    assert!(error.contains("Failed to parse bindings module"), "{error}");
  }
}
