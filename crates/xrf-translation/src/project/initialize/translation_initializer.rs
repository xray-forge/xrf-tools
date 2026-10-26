use std::path::Path;
use std::time::Instant;

use walkdir::{DirEntry, WalkDir};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::json::translation_json_reader::TranslationJsonReader;
use crate::json::translation_json_render::TranslationJsonRender;
use crate::json::translation_source_name::TranslationSourceName;
use crate::language::translation_language::TranslationLanguage;
use crate::project::initialize::translation_initialize_options::TranslationInitializeOptions;
use crate::project::initialize::translation_initialize_result::TranslationInitializeResult;
use crate::text::translation_json::TranslationJson;

/// Gives every id an explicit `null` for each language the project ships but the record has no text for.
pub struct TranslationInitializer;

impl TranslationInitializer {
  /// Scaffold every language into the source or tree at `path`.
  ///
  /// # Errors
  ///
  /// The same as [`Self::initialize_opt`].
  pub fn initialize(path: &Path) -> XrfResult<TranslationInitializeResult> {
    Self::initialize_opt(path, TranslationInitializeOptions::default())
  }

  /// Scaffold every language into the source or tree at `path`, reporting through `options`.
  ///
  /// A file and a directory are both accepted, because a caller holding one path should not have to ask which it is
  /// before choosing an entry point — that decision belongs to the operation that knows what it does with either.
  ///
  /// Only a source that gained something is rewritten, so running this over an already complete project writes
  /// nothing. It intentionally neither reorders nor reformats: that is `TranslationFormatter`'s job, and is why the two
  /// commands exist separately.
  ///
  /// # Errors
  ///
  /// Returns a read error when a directory cannot be walked, a parsing error for a source that cannot be read, and an
  /// IO error when one cannot be replaced.
  pub fn initialize_opt(path: &Path, options: TranslationInitializeOptions) -> XrfResult<TranslationInitializeResult> {
    let started_at: Instant = Instant::now();

    let mut result: TranslationInitializeResult = if path.is_dir() {
      Self::initialize_dir(path, &options)?
    } else {
      Self::initialize_file(path, &options)?
    };

    result.duration = started_at.elapsed();

    log::info!(
      "Initialized {} of {} translation source(s) at {} in {}, {} key(s) added",
      result.files_initialized,
      result.files_read,
      format_path(path),
      xrf_utils::format_duration(result.duration),
      result.keys_added
    );

    Ok(result)
  }

  /// Walk a tree, initializing every source in it.
  fn initialize_dir(dir: &Path, options: &TranslationInitializeOptions) -> XrfResult<TranslationInitializeResult> {
    xrf_output::info!(options.output, "Initializing dir {}", format_path(dir));

    let mut result: TranslationInitializeResult = TranslationInitializeResult::new();

    for entry in WalkDir::new(dir).sort_by_file_name() {
      let entry: DirEntry = entry.map_err(|error| {
        XrfError::new_read_error(format!(
          "Failed to walk translation directory '{}': {error}",
          format_path(dir)
        ))
      })?;

      if entry.path().is_file() {
        result.merge(&Self::initialize_file(entry.path(), options)?);
      }
    }

    Ok(result)
  }

  /// Initialize one source, passing over anything that is not a multi-language JSON.
  fn initialize_file(path: &Path, options: &TranslationInitializeOptions) -> XrfResult<TranslationInitializeResult> {
    // Through the shared name parser rather than comparing an extension, so this recognises the same names the reader
    // does; see `TranslationSourceName` for the case that made them drift.
    if !TranslationSourceName::is_path(path) {
      log::info!("Skip file {}", format_path(path));
      xrf_output::info!(options.output, "Skip file {}", format_path(path));

      return Ok(TranslationInitializeResult {
        files_skipped: 1,
        ..TranslationInitializeResult::new()
      });
    }

    let mut parsed: TranslationJson = TranslationJsonReader::read(path)?;
    let mut keys_added: u32 = 0;

    for (id, entry) in &mut parsed {
      for language in TranslationLanguage::LANGUAGES {
        let code: String = language.to_string();

        if !entry.contains_key(&code) {
          keys_added += 1;

          log::info!("Initializing missing key: {id} - {code}");
          xrf_output::info!(options.output, "Initializing missing key: {id} - {code}");

          entry.insert(code, None);
        }
      }
    }

    // Through the canonical writer, which is also how the trailing newline this used to drop survives: it serialized
    // with `to_vec_pretty` and appended nothing, so every file it touched lost the one it came with.
    if keys_added > 0 {
      TranslationJsonRender::write_document(path, &parsed, None)?;
    }

    Ok(TranslationInitializeResult {
      files_read: 1,
      files_initialized: u32::from(keys_added > 0),
      keys_added,
      ..TranslationInitializeResult::new()
    })
  }
}
