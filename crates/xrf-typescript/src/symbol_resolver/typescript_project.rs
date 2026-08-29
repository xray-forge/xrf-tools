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
    let unresolved: PathBuf = if specifier.starts_with('.') {
      source_file.parent()?.join(specifier)
    } else {
      self.resolve_path_alias(specifier)?
    };

    find_typescript_module(&unresolved)
  }

  /// Resolve one configured TypeScript path alias.
  fn resolve_path_alias(&self, specifier: &str) -> Option<PathBuf> {
    let base_url: PathBuf = self
      .compiler_options
      .base_url
      .as_deref()
      .map(|value| self.root.join(value))
      .unwrap_or_else(|| self.root.clone());

    for (pattern, targets) in &self.compiler_options.paths {
      let Some(capture) = path_alias_capture(pattern, specifier) else {
        continue;
      };
      let target: &str = targets.first()?;

      return Some(base_url.join(target.replace('*', capture)));
    }

    None
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

/// Find a TypeScript source file for an unresolved module path.
fn find_typescript_module(path: &Path) -> Option<PathBuf> {
  let direct: PathBuf = path.to_path_buf();
  let file: PathBuf = path.with_extension("ts");
  let index: PathBuf = path.join("index.ts");

  [direct, file, index].into_iter().find(|candidate| candidate.is_file())
}
