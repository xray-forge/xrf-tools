use std::collections::HashMap;
use std::fs::{self, File};
use std::path::{Component, Path, PathBuf};

use xrf_error::{XrfError, XrfResult};
use xrf_utils::{format_path, to_portable_path_string};
use xrf_vfs::XrayRoots;

use crate::language::TranslationLanguage;
use crate::project::build::options::ProjectBuildOptions;
use crate::source_file_name::parse_json_source_stem;
use crate::xml;

/// Open the file a source builds to, creating the directories above it.
///
/// # Errors
///
/// Returns an IO error when the target cannot be created.
pub(crate) fn prepare_target_file(target: &Path, options: &ProjectBuildOptions) -> XrfResult<File> {
  xrf_output::verbose!(options.output, "Writing file {}", format_path(target));

  let target_parent: &Path = target.parent().ok_or_else(|| {
    XrfError::new_invalid_error(format!("Translation XML target has no parent: {}", format_path(target)))
  })?;

  fs::create_dir_all(target_parent)?;

  Ok(File::options().write(true).create(true).truncate(true).open(target)?)
}

/// Where one source lands, which is `<output>/<language>/<source path>.xml`.
///
/// The source is named by the logical path the VFS resolved it under, so the built tree mirrors the
/// source tree whatever host the sources came off. The destination directory identifies the language,
/// so the JSON stem is all the name that is needed.
///
/// # Errors
///
/// Returns an invalid error when the source is not a name this builds from.
pub(crate) fn target_path(source: &str, destination: &Path, language: &TranslationLanguage) -> XrfResult<PathBuf> {
  let mut relative: PathBuf = PathBuf::new();

  // Component by component, because a logical path is `\`-separated whatever the host uses. A host
  // path from the single-file entry point splits on its own separator too, which `Path` handles.
  for component in source.split(['\\', '/']).filter(|part| !part.is_empty()) {
    relative.push(component);
  }

  let stem: String = relative
    .file_name()
    .and_then(parse_json_source_stem)
    .ok_or_else(|| {
      XrfError::new_invalid_error(format!("Translation source '{source}' is not a name this builds from"))
    })?
    .to_owned();

  relative.set_file_name(format!("{stem}.{}", xml::FILE_EXTENSION));

  Ok(destination.join(language.to_string()).join(relative))
}

/// Refuse a build where two sources would write the same file.
///
/// # Errors
///
/// Returns an invalid error naming both sources and the target they collide on.
pub(crate) fn validate_targets(sources: &[String], options: &ProjectBuildOptions) -> XrfResult {
  let mut target_sources: HashMap<String, &str> = HashMap::new();

  for source in sources {
    for language in target_languages_for_source(source, options) {
      let target: PathBuf = target_path(source, &options.output_dir, &language)?;
      let target_key: String = to_portable_path_string(&target).to_lowercase();

      if let Some(existing_source) = target_sources.insert(target_key, source)
        && existing_source != source
      {
        return Err(XrfError::new_invalid_error(format!(
          "Translation sources '{}' and '{}' both build to '{}'",
          existing_source,
          source,
          format_path(&target),
        )));
      }
    }
  }

  Ok(())
}

/// Every language one source will be written for, which decides what its targets collide with.
///
/// A JSON source carries all of them, so it builds for whichever the run asked for - one, or all
/// eight. Anything that is not a source builds for none.
pub(crate) fn target_languages_for_source(source: &str, options: &ProjectBuildOptions) -> Vec<TranslationLanguage> {
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
/// The hazard this was written for is gone: unsuffixed XML used to be a source, so a built
/// `example.xml` could overwrite the `example.xml` it came from. Sources are JSON and output is XML
/// now, so a build cannot clobber its own input by name. What is left is that filling an authored
/// tree with 272 generated files is still nobody's intention, which is reason enough to keep it.
///
/// Every loose root is checked, since a layered build reads from all of them. An archived root has no
/// host path and nothing can be written inside it anyway.
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
      format_path(output),
      format_path(source),
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

/// Whether `path` sits at or below `parent`, compared component by component.
///
/// Case-folded for the same reason as the collision key: the only caller refuses on a match, so
/// answering yes when a case-sensitive host would have said no costs a build that has to be re-pointed,
/// while answering no where Windows would resolve them to the same directory lets a build overwrite the
/// sources it was built from. Components rather than string prefixes, so `src_backup` is not inside
/// `src`.
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
