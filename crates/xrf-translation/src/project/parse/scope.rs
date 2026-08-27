use xrf_error::{XrfError, XrfResult};
use xrf_vfs::{XrayDirectoryListing, XrayVfs};

use crate::language::TranslationLanguage;
use crate::project::constants::MAP_DESC_DIRECTORY;
use crate::project::descriptor::TranslationProjectMode;

/// Where under the mounted roots one language's raw string tables actually sit.
///
/// Three steps, in order, because a caller reasonably names any of three things: the root of an
/// installation or mod, the text root inside it, or the language directory itself.
///
/// 1. An explicit prefix is obeyed and nothing is searched for.
/// 2. Otherwise the gamedata text prefix is taken when the roots have one, since that is where a
///    shipped tree keeps its tables.
/// 3. Then, if the resolved directory holds a child named exactly the declared language, the scope
///    descends into it. This is `CStringTable::FillLanguageToken`'s own rule rather than an invention
///    of ours: a language directory below the text root is how gamedata spells the language.
///
/// # Errors
///
/// Returns an invalid error when the resolved scope still holds directories named for languages other
/// than the declared one. That is the one dangerous misuse: pointing at `configs\text` while naming a
/// single language would otherwise read every language's files and file them all under that one name,
/// silently — 24,802 entries in two languages, on an Anomaly-sized tree.
pub(crate) fn resolve(
  vfs: &XrayVfs,
  prefix: Option<&str>,
  language: TranslationLanguage,
) -> XrfResult<ResolvedParseScope> {
  let language_name: String = language.to_string();

  let mut prefix: String = match prefix {
    Some(prefix) => prefix.to_owned(),
    None => default_prefix(vfs),
  };

  if has_child_directory(vfs, &prefix, &language_name) {
    prefix = join(&prefix, &language_name);
  }

  let foreign: Vec<String> = foreign_language_directories(vfs, &prefix, &language_name);

  if !foreign.is_empty() {
    return Err(XrfError::new_invalid_error(format!(
      "Translations at '{}' hold other languages ({}), so reading them all as '{language_name}' would file \
       them under the wrong language. Name one language's directory with --prefix, or run once per language.",
      display_prefix(&prefix),
      foreign.join(", "),
    )));
  }

  Ok(ResolvedParseScope { prefix })
}

/// The prefix one import run reads under, once every fallback has been applied.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ResolvedParseScope {
  prefix: String,
}

impl ResolvedParseScope {
  pub(crate) fn prefix(&self) -> &str {
    &self.prefix
  }

  /// How to name this scope to somebody reading a log line, where an empty prefix means the root.
  pub(crate) fn describe(&self) -> String {
    display_prefix(&self.prefix)
  }
}

/// The gamedata text prefix when the roots have one, and the root itself otherwise.
///
/// Only taken when the directory actually exists, so a caller who mounted the text directory directly
/// is not sent looking for `configs\text` inside it.
fn default_prefix(vfs: &XrayVfs) -> String {
  let gamedata: &str = TranslationProjectMode::Gamedata.get_prefix();

  if vfs
    .list_children(gamedata)
    .is_ok_and(|listing: XrayDirectoryListing| !listing.is_empty())
  {
    return gamedata.to_owned();
  }

  String::new()
}

fn has_child_directory(vfs: &XrayVfs, prefix: &str, name: &str) -> bool {
  vfs.list_children(prefix).is_ok_and(|listing| {
    listing
      .directories
      .iter()
      .any(|directory| directory.eq_ignore_ascii_case(name))
  })
}

/// Directories under the scope named for a language that is not the one being read.
///
/// `map_desc` is excluded by name, exactly as the engine excludes it, so a tree carrying map
/// descriptions is not mistaken for one carrying a second language.
fn foreign_language_directories(vfs: &XrayVfs, prefix: &str, language_name: &str) -> Vec<String> {
  let Ok(listing) = vfs.list_children(prefix) else {
    return Vec::new();
  };

  listing
    .directories
    .into_iter()
    .filter(|directory| {
      !directory.eq_ignore_ascii_case(MAP_DESC_DIRECTORY)
        && !directory.eq_ignore_ascii_case(language_name)
        && TranslationLanguage::from_str_single(directory).is_ok()
    })
    .collect()
}

fn join(prefix: &str, name: &str) -> String {
  if prefix.is_empty() {
    return name.to_owned();
  }

  format!("{prefix}\\{name}")
}

fn display_prefix(prefix: &str) -> String {
  if prefix.is_empty() {
    String::from("<root>")
  } else {
    prefix.to_owned()
  }
}
