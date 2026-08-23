use std::path::{Path, PathBuf};

use indexmap::IndexMap;
use walkdir::WalkDir;
use xrf_error::{XrfError, XrfResult};

use crate::file::DialogFile;
use crate::project::descriptor::{DialogDescriptor, DialogFileDescriptor, DialogFinding, DialogProjectDescriptor};
use crate::project::layout::{normalize, relative};
use crate::project::mode::DialogProjectMode;
use crate::project::roots::{DialogProjectOverrides, DialogProjectRoots};

const XML_EXTENSION: &str = "xml";

/// Filename prefix that marks a file as dialog data.
///
/// A gameplay directory holds `info_*.xml` and `npc_profile*.xml` beside the dialogs, so the
/// extension alone would sweep files this reader does not model.
const DIALOG_FILE_PREFIX: &str = "dialog";

/// One file the project holds, parsed, with where it came from.
#[derive(Debug)]
pub struct DialogProjectFile {
  path: PathBuf,
  relative_path: String,
  file: DialogFile,
}

impl DialogProjectFile {
  pub fn get_path(&self) -> &Path {
    &self.path
  }

  /// Path relative to the dialogs root, separator-normalised, which is how the index keys it.
  pub fn get_relative_path(&self) -> &str {
    &self.relative_path
  }

  pub fn get_file(&self) -> &DialogFile {
    &self.file
  }
}

/// An open dialog project: both roots, and every dialog file under the first of them.
///
/// The parsed files are kept rather than re-read per selection, because their spans are what a later
/// edit splices and re-parsing would hand out ranges into a string nobody still holds.
#[derive(Debug)]
pub struct DialogProject {
  mode: DialogProjectMode,
  root: PathBuf,
  roots: DialogProjectRoots,
  files: Vec<DialogProjectFile>,
  findings: Vec<DialogFinding>,
}

impl DialogProject {
  /// Open a project, reading every dialog file its dialogs root holds.
  ///
  /// A file that cannot be read becomes a finding and the project still opens: refusing the whole
  /// tree over one bad file would make the editor unable to reach the file you opened it to fix.
  ///
  /// # Errors
  ///
  /// Returns a not-found error when the dialogs root is absent or holds no dialog files. Both mean
  /// the caller named the wrong place, and answering with an empty project would hide that.
  pub fn open(root: &Path, mode: DialogProjectMode, overrides: &DialogProjectOverrides) -> XrfResult<Self> {
    let roots: DialogProjectRoots = DialogProjectRoots::resolve(root, mode, overrides);
    let dialogs_root: &Path = roots.get_dialogs();

    if !dialogs_root.exists() {
      return Err(XrfError::new_not_found_error(format!(
        "Dialogs root does not exist: {}",
        dialogs_root.display()
      )));
    }

    let paths: Vec<PathBuf> = Self::list_dialog_paths(dialogs_root);

    if paths.is_empty() {
      return Err(XrfError::new_not_found_error(format!(
        "No dialog files under {}",
        dialogs_root.display()
      )));
    }

    let mut files: Vec<DialogProjectFile> = Vec::new();
    let mut findings: Vec<DialogFinding> = Vec::new();

    for path in paths {
      let relative_path: String = relative(dialogs_root, &path);

      match DialogFile::read_from_path(&path) {
        Ok(file) => {
          for issue in file.get_issues() {
            findings.push(DialogFinding::new(
              "dialog.schema",
              Some(relative_path.clone()),
              issue.to_string(),
            ));
          }

          files.push(DialogProjectFile {
            path,
            relative_path,
            file,
          });
        }
        Err(error) => findings.push(DialogFinding::new(
          "dialog.unreadable",
          Some(relative_path),
          error.to_string(),
        )),
      }
    }

    Ok(Self {
      mode,
      root: root.to_path_buf(),
      roots,
      files,
      findings,
    })
  }

  /// Every dialog file under a root, in a stable order.
  ///
  /// Sorted because `WalkDir` follows the filesystem, and an index is only comparable across runs and
  /// machines if it does not.
  pub fn list_dialog_paths(root: &Path) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = WalkDir::new(root)
      .into_iter()
      .filter_map(Result::ok)
      .filter(|entry| entry.file_type().is_file())
      .map(|entry| entry.into_path())
      .filter(|path| Self::is_dialog_path(path))
      .collect();

    paths.sort();

    paths
  }

  /// Whether a path names dialog data, by extension and filename prefix.
  pub fn is_dialog_path(path: &Path) -> bool {
    let is_xml: bool = path
      .extension()
      .is_some_and(|extension| extension.eq_ignore_ascii_case(XML_EXTENSION));

    is_xml
      && path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.to_ascii_lowercase().starts_with(DIALOG_FILE_PREFIX))
  }

  pub fn get_mode(&self) -> DialogProjectMode {
    self.mode
  }

  pub fn get_roots(&self) -> &DialogProjectRoots {
    &self.roots
  }

  pub fn get_files(&self) -> &[DialogProjectFile] {
    &self.files
  }

  pub fn get_findings(&self) -> &[DialogFinding] {
    &self.findings
  }

  /// The file at a path relative to the dialogs root.
  pub fn find_file(&self, relative_path: &str) -> Option<&DialogProjectFile> {
    self.files.iter().find(|file| file.get_relative_path() == relative_path)
  }

  /// Total dialogs across every file the project read.
  pub fn sum_dialogs(&self) -> usize {
    self.files.iter().map(|file| file.get_file().get_dialogs().len()).sum()
  }

  /// The project as it crosses the wire: the index, not the phrases.
  pub fn describe(&self) -> DialogProjectDescriptor {
    let mut files: IndexMap<String, DialogFileDescriptor> = IndexMap::new();

    for entry in &self.files {
      files.insert(
        entry.get_relative_path().to_owned(),
        DialogFileDescriptor {
          path: normalize(entry.get_path()),
          encoding: String::from(entry.get_file().get_encoding().name()),
          dialogs: entry
            .get_file()
            .get_dialogs()
            .iter()
            .map(|dialog| DialogDescriptor {
              id: dialog.get_id().to_owned(),
              phrases: dialog.get_phrases().len(),
              priority: dialog.get_priority(),
            })
            .collect(),
        },
      );
    }

    DialogProjectDescriptor {
      mode: self.mode,
      root: normalize(&self.root),
      dialogs_root: normalize(self.roots.get_dialogs()),
      translations_root: normalize(self.roots.get_translations()),
      files,
      findings: self.findings.clone(),
    }
  }
}
