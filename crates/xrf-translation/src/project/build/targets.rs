use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use xrf_error::{XrfError, XrfResult};
use xrf_extension::XrayExtension;
use xrf_utils::{HostFileName, HostRelativePath, format_path, to_absolute_path, to_portable_path_string};
use xrf_vfs::XrayRoots;

use crate::language::TranslationLanguage;
use crate::project::build::translation_build_options::TranslationBuildOptions;
use crate::source_file_name::parse_json_source_stem;

/// Where one source lands below the output directory: `<language>/<source name>.xml`.
///
/// # Errors
///
/// Returns an invalid error when the source is not a name this builds from, or would leave the output directory.
pub(crate) fn target_path(source: &str, language: &TranslationLanguage) -> XrfResult<HostRelativePath> {
  let relative: HostRelativePath = HostRelativePath::new(source, "Translation source")?;

  let stem: String = relative
    .file_name()
    .and_then(parse_json_source_stem)
    .ok_or_else(|| {
      XrfError::new_invalid_error(format!("Translation source '{source}' is not a name this builds from"))
    })?
    .to_owned();

  let file: HostFileName = HostFileName::new(&stem, "Translation source")?.with_extension(XrayExtension::Xml.as_str());

  Ok(
    HostRelativePath::new(&language.to_string(), "Translation language")?
      .join_path(&relative.parent().unwrap_or_default())
      .join(&file),
  )
}

/// Refuse a build where two sources would write the same file.
///
/// # Errors
///
/// Returns an invalid error naming both sources and the target they collide on.
pub(crate) fn validate_targets(sources: &[String], options: &TranslationBuildOptions) -> XrfResult {
  let mut target_sources: HashMap<String, &str> = HashMap::new();

  for source in sources {
    for language in target_languages_for_source(source, options) {
      let target: HostRelativePath = target_path(source, &language)?;
      let target_key: String = to_portable_path_string(target.as_path()).to_lowercase();

      if let Some(existing_source) = target_sources.insert(target_key, source)
        && existing_source != source
      {
        return Err(XrfError::new_invalid_error(format!(
          "Translation sources '{}' and '{}' both build to '{}'",
          existing_source,
          source,
          format_path(&options.output_dir.join(&target)),
        )));
      }
    }
  }

  Ok(())
}

/// Every language one source will be written for, which decides what its targets collide with.
pub(crate) fn target_languages_for_source(source: &str, options: &TranslationBuildOptions) -> Vec<TranslationLanguage> {
  if Path::new(source).file_name().and_then(parse_json_source_stem).is_none() {
    return Vec::new();
  }

  if options.language == TranslationLanguage::All {
    TranslationLanguage::get_all()
  } else {
    vec![options.language]
  }
}

/// Refuse to write build output inside any of the trees it was built from.
///
/// # Errors
///
/// Returns an invalid error naming the root the output would sit inside.
pub(crate) fn ensure_output_outside_roots(roots: &XrayRoots, output: &Path) -> XrfResult {
  for root in &roots.roots {
    let source: &Path = &root.path;

    if !source.is_dir() {
      continue;
    }

    ensure_output_outside_source(source, output)?;
  }

  Ok(())
}

/// Refuse to write build output into the sources it was built from.
///
/// # Errors
///
/// Returns an invalid error when the output directory is inside the source directory.
pub(crate) fn ensure_output_outside_source(source: &Path, output: &Path) -> XrfResult {
  let source_lexical: PathBuf = to_absolute_path(source)?;
  let output_lexical: PathBuf = to_absolute_path(output)?;
  let source_resolved: PathBuf = fs::canonicalize(source).unwrap_or_else(|_| source_lexical.clone());
  let output_resolved: PathBuf = fs::canonicalize(output).unwrap_or_else(|_| output_lexical.clone());

  if path_is_within(&output_lexical, &source_lexical) || path_is_within(&output_resolved, &source_resolved) {
    return Err(XrfError::new_invalid_error(format!(
      "Translation output '{}' must be outside source directory '{}'",
      format_path(output),
      format_path(source),
    )));
  }

  Ok(())
}

/// Whether `path` sits at or below `parent`, compared component by component.
fn path_is_within(path: &Path, parent: &Path) -> bool {
  let path_components: Vec<String> = path
    .components()
    .map(|component| component.as_os_str().to_string_lossy().to_lowercase())
    .collect();
  let parent_components: Vec<String> = parent
    .components()
    .map(|component| component.as_os_str().to_string_lossy().to_lowercase())
    .collect();

  path_components.len() >= parent_components.len()
    && path_components
      .iter()
      .zip(parent_components.iter())
      .all(|(path_component, parent_component)| path_component == parent_component)
}
