use std::collections::HashMap;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use xrf_error::{XrfError, XrfResult};

use crate::host_path::format_path;

/// The files one run writes into a directory, held to the workspace's force rule: unforced, the run refuses before any
/// work when one of them is already there; forced, it replaces them and removes those of its own it no longer writes.
///
/// A run names its files ([`Self::with_file`], below the directory, separated by `/` or `\`) and the families it owns
/// ([`Self::with_family`], by name prefix, as a compile owns every `lmap#` page). Names are compared without case, as
/// the game and Windows read them, so a run on a case-sensitive file system meets the same refusal it would on Windows.
#[derive(Clone, Debug)]
pub struct OutputFiles {
  directory: PathBuf,
  /// Each file's path below the directory, by component, folded.
  files: Vec<Vec<String>>,
  families: Vec<OutputFamily>,
}

/// Files a run owns by name prefix in one directory below its own.
#[derive(Clone, Debug)]
struct OutputFamily {
  /// The directory, by component, folded; empty for the run's directory itself.
  directory: Vec<String>,
  /// The prefix of each member's name, folded.
  prefix: String,
}

impl OutputFiles {
  /// How many of the files found a refusal names before counting the rest.
  const NAMED_IN_REFUSAL: usize = 5;

  pub fn new(directory: &Path) -> Self {
    Self {
      directory: directory.to_path_buf(),
      files: Vec::new(),
      families: Vec::new(),
    }
  }

  /// What a run writing one file or directory, `path`, owns: that alone, in the directory above it.
  pub fn of_path(path: &Path) -> Self {
    let output: Self = Self::new(path.parent().unwrap_or_else(|| Path::new("")));

    match path.file_name() {
      Some(name) => output.with_file(&name.to_string_lossy()),
      None => output,
    }
  }

  pub fn get_directory(&self) -> &Path {
    &self.directory
  }

  /// Adds a file the run writes, `name` a path below the directory.
  #[must_use]
  pub fn with_file(mut self, name: &str) -> Self {
    self.files.push(Self::split(name));
    self
  }

  /// Adds every file the run writes, each a path below the directory.
  #[must_use]
  pub fn with_files<'a>(self, names: impl IntoIterator<Item = &'a str>) -> Self {
    names.into_iter().fold(self, Self::with_file)
  }

  /// Adds a family of files the run owns: every one whose name starts with `prefix`'s last component, in the directory
  /// its other components name below the run's (`meshes/brkbl#` for every breakable part under `meshes`); every one in
  /// that directory where `prefix` ends in a separator (`test_level/`).
  #[must_use]
  pub fn with_family(mut self, prefix: &str) -> Self {
    let mut directory: Vec<String> = Self::split(prefix);
    let prefix: String = if prefix.ends_with(['/', '\\']) {
      String::new()
    } else {
      directory.pop().unwrap_or_default()
    };

    self.families.push(OutputFamily { directory, prefix });
    self
  }

  /// Refuses, unless forced, when any file the run writes is already there.
  ///
  /// # Errors
  ///
  /// Returns an already-exists error naming the files found, or an IO error when the directory cannot be listed.
  pub fn assert_free(&self, is_forced: bool) -> XrfResult {
    if is_forced {
      return Ok(());
    }

    let found: Vec<PathBuf> = self.find_existing()?;

    if found.is_empty() {
      return Ok(());
    }

    Err(Self::new_replace_refusal(
      &format_path(&self.directory).to_string(),
      &found
        .iter()
        .map(|path| format_path(path.strip_prefix(&self.directory).unwrap_or(path)).to_string())
        .collect::<Vec<String>>(),
    ))
  }

  /// The refusal for an unforced run when `holder` already has the files named in `found`, for a run whose outputs do
  /// not sit in one directory. Callers detect this wording to ask before replacing.
  pub fn new_replace_refusal(holder: &str, found: &[String]) -> XrfError {
    let mut named: Vec<&str> = found.iter().take(Self::NAMED_IN_REFUSAL).map(String::as_str).collect();
    let more: String = format!("{} more", found.len().saturating_sub(Self::NAMED_IN_REFUSAL));

    if found.len() > Self::NAMED_IN_REFUSAL {
      named.push(&more);
    }

    XrfError::new_io_error(
      format!(
        "{holder} already holds what this run writes ({}); force it to replace them",
        named.join(", ")
      ),
      ErrorKind::AlreadyExists,
    )
  }

  /// Each file the run names ([`Self::with_file`]), in that order: the path the file system holds it under where it is
  /// there, matched without case; its families are not looked at.
  ///
  /// # Errors
  ///
  /// Returns an IO error when a directory on the way cannot be listed.
  pub fn find_each(&self) -> XrfResult<Vec<Option<PathBuf>>> {
    let mut listings: HashMap<PathBuf, HashMap<String, String>> = HashMap::new();

    self
      .files
      .iter()
      .map(|file| Self::find_file(&self.directory, file, &mut listings))
      .collect()
  }

  /// The files of the run already there, by the names the file system holds them under, in the order the run names
  /// them and then its families' members by name; none where the directory is missing.
  ///
  /// # Errors
  ///
  /// Returns an IO error when a directory on the way cannot be listed.
  pub fn find_existing(&self) -> XrfResult<Vec<PathBuf>> {
    let mut listings: HashMap<PathBuf, HashMap<String, String>> = HashMap::new();
    let mut found: Vec<PathBuf> = Vec::new();

    for file in &self.files {
      if let Some(path) = Self::find_file(&self.directory, file, &mut listings)? {
        found.push(path);
      }
    }

    for family in &self.families {
      let Some(directory) = Self::find_file(&self.directory, &family.directory, &mut listings)? else {
        continue;
      };
      let mut members: Vec<(String, PathBuf)> = Self::list(&directory, &mut listings)?
        .iter()
        .filter(|(folded, _)| folded.starts_with(family.prefix.as_str()))
        .map(|(folded, name)| (folded.clone(), directory.join(name)))
        .filter(|(_, path)| !found.contains(path))
        .collect();

      members.sort();
      found.extend(members.into_iter().map(|(_, path)| path));
    }

    Ok(found)
  }

  /// Removes the run's files that are there but not among `written`, the paths it wrote this time, compared below the
  /// directory without case: what an earlier, larger output leaves. Answers what it removed.
  ///
  /// # Errors
  ///
  /// Returns an invalid error, removing nothing, when a written path is not below the directory as spelled here; an IO
  /// error when a directory cannot be listed or a stale file cannot be removed.
  pub fn remove_stale(&self, written: &[PathBuf]) -> XrfResult<Vec<PathBuf>> {
    self.remove_stale_where(written, |_| true)
  }

  /// [`Self::remove_stale`], removing only the stale files `is_removable` takes: where a family's names alone would also
  /// take files the run did not make, it proves which it made. Every stale file is judged before the first is removed,
  /// so removing one does not change the answer for another.
  ///
  /// # Errors
  ///
  /// As [`Self::remove_stale`].
  pub fn remove_stale_where(
    &self,
    written: &[PathBuf],
    is_removable: impl Fn(&Path) -> bool,
  ) -> XrfResult<Vec<PathBuf>> {
    let written: Vec<String> = written
      .iter()
      .map(|path| self.fold_below(path))
      .collect::<XrfResult<_>>()?;
    let mut stale: Vec<PathBuf> = Vec::new();

    for path in self.find_existing()? {
      if !written.contains(&self.fold_below(&path)?) && path.is_file() && is_removable(&path) {
        stale.push(path);
      }
    }

    for path in &stale {
      fs::remove_file(path)?;
    }

    Ok(stale)
  }

  /// The path a file's folded components name below `directory`, each matched without case; none when one is
  /// missing.
  fn find_file(
    directory: &Path,
    components: &[String],
    listings: &mut HashMap<PathBuf, HashMap<String, String>>,
  ) -> XrfResult<Option<PathBuf>> {
    let mut path: PathBuf = directory.to_path_buf();

    for component in components {
      let Some(name) = Self::list(&path, listings)?.get(component).cloned() else {
        return Ok(None);
      };

      path.push(name);
    }

    Ok(Some(path))
  }

  /// The entries of `directory`, folded name to the name the file system holds, listed once a run; empty where it is
  /// missing or not a directory.
  fn list<'a>(
    directory: &Path,
    listings: &'a mut HashMap<PathBuf, HashMap<String, String>>,
  ) -> XrfResult<&'a HashMap<String, String>> {
    if !listings.contains_key(directory) {
      let mut entries: HashMap<String, String> = HashMap::new();
      // A bare file name's directory is empty, which names the working directory, though `read_dir` finds none there.
      let listed: &Path = if directory.as_os_str().is_empty() {
        Path::new(".")
      } else {
        directory
      };

      match fs::read_dir(listed) {
        Ok(read) => {
          for entry in read {
            let name: String = entry?.file_name().to_string_lossy().into_owned();

            entries.insert(Self::fold(&name), name);
          }
        }
        Err(error) if matches!(error.kind(), ErrorKind::NotFound | ErrorKind::NotADirectory) => {}
        Err(error) => return Err(error.into()),
      }

      listings.insert(directory.to_path_buf(), entries);
    }

    Ok(&listings[directory])
  }

  /// A path below the directory, by component, folded.
  fn split(path: &str) -> Vec<String> {
    path
      .split(['/', '\\'])
      .filter(|component| !component.is_empty())
      .map(Self::fold)
      .collect()
  }

  fn fold(name: &str) -> String {
    name.to_lowercase()
  }

  /// A path below the directory, folded with one separator.
  fn fold_below(&self, path: &Path) -> XrfResult<String> {
    let below: &Path = path.strip_prefix(&self.directory).map_err(|_| {
      XrfError::new_invalid_error(format!(
        "{} is not below {}, the directory whose stale files a run removes",
        format_path(path),
        format_path(&self.directory)
      ))
    })?;

    Ok(Self::fold(&below.to_string_lossy().replace('\\', "/")))
  }
}
