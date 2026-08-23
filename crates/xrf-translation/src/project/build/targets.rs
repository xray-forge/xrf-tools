use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::{self, File};
use std::path::{Component, Path, PathBuf};

use xrf_error::{XrfError, XrfResult};

use crate::json;
use crate::language::TranslationLanguage;
use crate::project::build::options::ProjectBuildOptions;
use crate::xml;

/// Open the file a source builds to, creating the directories above it.
///
/// # Errors
///
/// Returns an invalid error when the source sits outside the project root, and an IO error when the
/// target cannot be created.
pub(crate) fn prepare_target_file<P1: AsRef<Path>, P2: AsRef<Path>>(
  path: &P1,
  destination: &P2,
  language: &TranslationLanguage,
  options: &ProjectBuildOptions,
) -> XrfResult<File> {
  let target: PathBuf = target_path(path.as_ref(), destination.as_ref(), language, options)?;

  xrf_output::verbose!(options.output, "Writing file ({}) {}", language, target.display());

  let target_parent: &Path = target.parent().ok_or_else(|| {
    XrfError::new_invalid_error(format!("Translation XML target has no parent: {}", target.display()))
  })?;

  fs::create_dir_all(target_parent)?;

  Ok(File::options().write(true).create(true).truncate(true).open(target)?)
}

/// Where one source lands, which is `<output>/<language>/<path relative to the root>.xml`.
///
/// # Errors
///
/// Returns an invalid error when the source is outside the project root or has no file name.
pub(crate) fn target_path(
  source: &Path,
  destination: &Path,
  language: &TranslationLanguage,
  options: &ProjectBuildOptions,
) -> XrfResult<PathBuf> {
  let relative_source: PathBuf = if source == options.path {
    PathBuf::from(source.file_name().ok_or_else(|| {
      XrfError::new_invalid_error(format!("Translation source has no file name: {}", source.display()))
    })?)
  } else {
    source.strip_prefix(&options.path).map(PathBuf::from).map_err(|_| {
      XrfError::new_invalid_error(format!(
        "Translation source '{}' is outside project root '{}'",
        source.display(),
        options.path.display(),
      ))
    })?
  };

  Ok(
    destination
      .join(language.to_string())
      .join(relative_source)
      .with_extension(xml::FILE_EXTENSION),
  )
}

/// Refuse a build where two sources would write the same file.
///
/// # Errors
///
/// Returns an invalid error naming both sources and the target they collide on.
pub(crate) fn validate_targets(source_files: &[PathBuf], options: &ProjectBuildOptions) -> XrfResult {
  let mut target_sources: HashMap<String, &Path> = HashMap::new();

  for source in source_files {
    for language in target_languages_for_source(source, options) {
      let target: PathBuf = target_path(source, &options.output_dir, &language, options)?;
      let target_key: String = target.to_string_lossy().replace('\\', "/").to_lowercase();

      if let Some(existing_source) = target_sources.insert(target_key, source)
        && existing_source != source
      {
        return Err(XrfError::new_invalid_error(format!(
          "Translation sources '{}' and '{}' both build to '{}'",
          existing_source.display(),
          source.display(),
          target.display(),
        )));
      }
    }
  }

  Ok(())
}

pub(crate) fn target_languages_for_source(path: &Path, options: &ProjectBuildOptions) -> Vec<TranslationLanguage> {
  match path.extension().and_then(OsStr::to_str) {
    Some(json::FILE_EXTENSION) => {
      if options.language == TranslationLanguage::All {
        TranslationLanguage::get_all()
      } else {
        vec![options.language]
      }
    }
    Some(xml::FILE_EXTENSION) => match path
      .file_name()
      .and_then(|it| it.to_str())
      .and_then(TranslationLanguage::from_file_name)
    {
      Some(locale) if options.language == TranslationLanguage::All || options.language == locale => vec![locale],
      Some(_) => Vec::new(),
      None if options.language == TranslationLanguage::All => TranslationLanguage::get_all(),
      None => vec![options.language],
    },
    _ => Vec::new(),
  }
}

/// Refuse to write build output into the sources it was built from.
///
/// Checked both lexically and against the resolved paths, so a symlink cannot smuggle the output back
/// inside the tree it would then overwrite.
///
/// # Errors
///
/// Returns an invalid error when the output directory is inside the source directory.
pub(crate) fn ensure_output_outside_source(source: &Path, output: &Path) -> XrfResult {
  let source_lexical: PathBuf = normalize_absolute_path(source)?;
  let output_lexical: PathBuf = normalize_absolute_path(output)?;
  let source_resolved: PathBuf = fs::canonicalize(source).unwrap_or_else(|_| source_lexical.clone());
  let output_resolved: PathBuf = fs::canonicalize(output).unwrap_or_else(|_| output_lexical.clone());

  if path_is_within(&output_lexical, &source_lexical) || path_is_within(&output_resolved, &source_resolved) {
    return Err(XrfError::new_invalid_error(format!(
      "Translation output '{}' must be outside source directory '{}'",
      output.display(),
      source.display(),
    )));
  }

  Ok(())
}

fn normalize_absolute_path(path: &Path) -> XrfResult<PathBuf> {
  let absolute: PathBuf = if path.is_absolute() {
    path.to_path_buf()
  } else {
    std::env::current_dir()?.join(path)
  };
  let mut normalized: PathBuf = PathBuf::new();

  for component in absolute.components() {
    match component {
      Component::CurDir => {}
      Component::ParentDir => {
        normalized.pop();
      }
      _ => normalized.push(component.as_os_str()),
    }
  }

  Ok(normalized)
}

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
