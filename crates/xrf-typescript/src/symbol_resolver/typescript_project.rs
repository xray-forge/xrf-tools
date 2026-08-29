use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

/// Resolves TypeScript modules using one project's compiler configuration.
pub struct TypeScriptProject {
  compiler_options: TypeScriptCompilerOptions,
  root: PathBuf,
}

impl TypeScriptProject {
  /// Discover the nearest TypeScript configuration from `source_root`.
  pub fn discover(source_root: &Path) -> XrfResult<Self> {
    let config_path: Option<PathBuf> = source_root
      .ancestors()
      .map(|directory| directory.join("tsconfig.json"))
      .find(|path| path.is_file());

    let Some(config_path) = config_path else {
      return Ok(Self {
        compiler_options: TypeScriptCompilerOptions::default(),
        root: source_root.into(),
      });
    };

    let config: TypeScriptConfig = read_typescript_config(&config_path)?;
    let root: PathBuf = config_path
      .parent()
      .expect("TypeScript configuration path has a parent")
      .into();

    Ok(Self {
      compiler_options: config.compiler_options,
      root,
    })
  }

  /// Resolve a module specifier from an importing TypeScript source file.
  pub fn resolve_module_path(&self, source_file: &Path, specifier: &str) -> Option<PathBuf> {
    if specifier.starts_with('.') {
      return find_typescript_module(&source_file.parent()?.join(specifier));
    }

    let (capture, targets) = self.best_path_alias(specifier)?;
    let base_url: PathBuf = self.base_url();

    // Configured target order is fallback order: the first target holding the module wins.
    targets
      .iter()
      .find_map(|target| find_typescript_module(&base_url.join(target.replace('*', capture))))
  }

  /// Select the most specific configured path alias matching `specifier`.
  ///
  /// Returns the wildcard capture and the chosen pattern's targets, which stay in configured order.
  /// Only an exact pattern and the wildcard extending it can rank equally, and the map's lexical
  /// order settles that pair on the exact one TypeScript also prefers.
  fn best_path_alias<'a>(&'a self, specifier: &'a str) -> Option<(&'a str, &'a [String])> {
    let mut best: Option<((usize, usize), &str, &[String])> = None;

    for (pattern, targets) in &self.compiler_options.paths {
      let Some(capture) = path_alias_capture(pattern, specifier) else {
        continue;
      };
      let specificity: (usize, usize) = path_alias_specificity(pattern);

      if best.is_none_or(|(current, ..)| specificity > current) {
        best = Some((specificity, capture, targets.as_slice()));
      }
    }

    best.map(|(_, capture, targets)| (capture, targets))
  }

  /// Resolve the directory that configured path-alias targets are relative to.
  fn base_url(&self) -> PathBuf {
    self
      .compiler_options
      .base_url
      .as_deref()
      .map(|value| self.root.join(value))
      .unwrap_or_else(|| self.root.clone())
  }
}

#[derive(Default, Deserialize)]
struct TypeScriptConfig {
  #[serde(default, rename = "compilerOptions")]
  compiler_options: TypeScriptCompilerOptions,
}

#[derive(Default, Deserialize)]
struct TypeScriptCompilerOptions {
  #[serde(default, rename = "baseUrl")]
  base_url: Option<String>,
  #[serde(default)]
  paths: BTreeMap<String, Vec<String>>,
}

/// Read the compiler options needed for native module resolution.
fn read_typescript_config(path: &Path) -> XrfResult<TypeScriptConfig> {
  let source: String = fs::read_to_string(path).map_err(|error| {
    XrfError::new_invalid_error(format!(
      "Failed to read TypeScript configuration '{}': {error}",
      format_path(path),
    ))
  })?;

  serde_json::from_str(&source).map_err(|error| {
    XrfError::new_invalid_error(format!(
      "Failed to parse TypeScript configuration '{}': {error}",
      format_path(path),
    ))
  })
}

/// Match a module specifier against a TypeScript path-alias pattern.
fn path_alias_capture<'a>(pattern: &str, specifier: &'a str) -> Option<&'a str> {
  let Some((prefix, suffix)) = pattern.split_once('*') else {
    return (pattern == specifier).then_some("");
  };

  specifier
    .strip_prefix(prefix)
    .and_then(|value| value.strip_suffix(suffix))
}

/// Rank one path-alias pattern by the literal text it fixes, most specific last.
///
/// TypeScript gives an import to the longest matching prefix rather than to the first configured
/// pattern, so `@/shared/*` owns what `@/*` also matches. Suffix length separates two patterns
/// sharing a prefix; a pattern without a wildcard fixes all of its text.
fn path_alias_specificity(pattern: &str) -> (usize, usize) {
  match pattern.split_once('*') {
    Some((prefix, suffix)) => (prefix.len(), suffix.len()),
    None => (pattern.len(), 0),
  }
}

/// Find a TypeScript source file for an unresolved module path.
fn find_typescript_module(path: &Path) -> Option<PathBuf> {
  let direct: PathBuf = path.to_path_buf();
  let file: PathBuf = path.with_extension("ts");
  let index: PathBuf = path.join("index.ts");

  [direct, file, index].into_iter().find(|candidate| candidate.is_file())
}

#[cfg(test)]
mod tests {
  use std::fs;
  use std::path::{Path, PathBuf};

  use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

  use super::TypeScriptProject;

  fn create_test_root(name: &str) -> PathBuf {
    let root: PathBuf = build_absolute_generated_test_resource_path(&format!("typescript-project/{name}"));

    fs::create_dir_all(&root).unwrap();

    root
  }

  fn write_source(root: &Path, name: &str, source: &str) {
    let path: PathBuf = root.join(name);

    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, source).unwrap();
  }

  fn resolve(root: &Path, specifier: &str) -> Option<PathBuf> {
    let source_file: PathBuf = root.join("src/declarations/externs.ts");

    TypeScriptProject::discover(source_file.parent().unwrap())
      .unwrap()
      .resolve_module_path(&source_file, specifier)
  }

  #[test]
  fn resolves_an_overlapping_alias_by_specificity_rather_than_map_order() {
    let root: PathBuf = create_test_root("overlapping");

    write_source(
      &root,
      "tsconfig.json",
      r#"{ "compilerOptions": { "paths": { "@/*": ["src/*"], "@/shared/*": ["shared/*"] } } }"#,
    );
    write_source(&root, "src/declarations/externs.ts", "export {};");
    write_source(&root, "src/shared/callbacks.ts", "export {};");
    write_source(&root, "src/other/callbacks.ts", "export {};");
    write_source(&root, "shared/callbacks.ts", "export {};");

    assert_eq!(
      resolve(&root, "@/shared/callbacks"),
      Some(root.join("shared/callbacks.ts"))
    );
    // The generic pattern still owns every specifier the specific one does not match.
    assert_eq!(
      resolve(&root, "@/other/callbacks"),
      Some(root.join("src/other/callbacks.ts"))
    );
  }

  #[test]
  fn settles_an_equally_specific_pattern_pair_on_the_exact_one() {
    let root: PathBuf = create_test_root("equal-specificity");

    write_source(
      &root,
      "tsconfig.json",
      r#"{ "compilerOptions": { "paths": { "@/shared": ["exact/handlers.ts"], "@/shared*": ["wildcard/handlers.ts"] } } }"#,
    );
    write_source(&root, "src/declarations/externs.ts", "export {};");
    write_source(&root, "exact/handlers.ts", "export {};");
    write_source(&root, "wildcard/handlers.ts", "export {};");

    assert_eq!(resolve(&root, "@/shared"), Some(root.join("exact/handlers.ts")));
  }

  #[test]
  fn falls_back_through_the_configured_targets_of_the_chosen_pattern() {
    let root: PathBuf = create_test_root("target-fallback");

    write_source(
      &root,
      "tsconfig.json",
      r#"{ "compilerOptions": { "baseUrl": "./roots", "paths": { "@/*": ["first/*", "second/*"] } } }"#,
    );
    write_source(&root, "src/declarations/externs.ts", "export {};");
    write_source(&root, "roots/first/shared.ts", "export {};");
    write_source(&root, "roots/second/shared.ts", "export {};");
    write_source(&root, "roots/second/only.ts", "export {};");

    assert_eq!(resolve(&root, "@/shared"), Some(root.join("roots/first/shared.ts")));
    assert_eq!(resolve(&root, "@/only"), Some(root.join("roots/second/only.ts")));
    assert_eq!(resolve(&root, "@/missing"), None);
  }

  #[test]
  fn resolves_a_relative_specifier_against_the_importing_file() {
    let root: PathBuf = create_test_root("relative");

    write_source(
      &root,
      "tsconfig.json",
      r#"{ "compilerOptions": { "paths": { "@/*": ["src/*"] } } }"#,
    );
    write_source(&root, "src/declarations/externs.ts", "export {};");
    write_source(&root, "src/declarations/helpers.ts", "export {};");

    assert_eq!(
      resolve(&root, "./helpers"),
      Some(root.join("src/declarations/helpers.ts"))
    );
    assert_eq!(resolve(&root, "./missing"), None);
  }
}
