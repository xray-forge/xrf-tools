use std::path::{Path, PathBuf};

use indexmap::IndexMap;
use xrf_error::{XrfError, XrfResult};
use xrf_utils::to_portable_path_string;
use xrf_vfs::{XrayAsset, XrayLogicalPath, XrayLookupScope, XrayRoots, XrayScopedVfs, XrayVfs};

use crate::dialog::Dialog;
use crate::file::DialogFile;
use crate::project::descriptor::{
  DialogDescriptor, DialogFileDescriptor, DialogFinding, DialogProjectDescriptor, DialogSummaryDescriptor,
};
use crate::project::layout::DialogProjectLayout;
use crate::project::mode::DialogProjectMode;

/// Filename prefix that marks a logical path as dialog data.
///
/// A gameplay directory holds `info_*.xml` and `npc_profile*.xml` beside the dialogs, so the
/// extension alone would sweep files this reader does not model.
const DIALOG_FILE_PREFIX: &str = "dialog";
const XML_SUFFIX: &str = ".xml";

/// One file the project holds, parsed, with where the engine found it.
#[derive(Debug)]
pub struct DialogProjectFile {
  logical_path: String,
  physical_path: Option<PathBuf>,
  file: DialogFile,
}

impl DialogProjectFile {
  /// The engine identity, which is how the project keys it.
  pub fn get_logical_path(&self) -> &str {
    &self.logical_path
  }

  /// The host path, when the winning mount is a loose directory.
  ///
  /// Absent for an archived winner, and that absence is the write guard: bytes inside a `.db` volume
  /// cannot be edited in place.
  pub fn get_physical_path(&self) -> Option<&Path> {
    self.physical_path.as_deref()
  }

  /// Whether an edit could write this file back.
  pub fn is_editable(&self) -> bool {
    self.physical_path.is_some()
  }

  pub fn get_file(&self) -> &DialogFile {
    &self.file
  }
}

/// An open dialog project: mounted roots, and every dialog file under its dialogs prefix.
///
/// Reads go through `xrf-vfs` rather than `std::fs`, because the engine does not see a disk. On a real
/// installation `configs\gameplay\dialogs.xml` comes out of `db\configs`, and a reader reaching for
/// the filesystem reports it absent instead of reading it.
///
/// The VFS is owned rather than borrowed, for the reason `LtxProject` owns its own: `XrayVfs` is not
/// `Clone`, and the project outlives any one lookup. The parsed files are kept too, because their
/// spans are what a later edit splices.
pub struct DialogProject {
  roots: XrayRoots,
  mode: DialogProjectMode,
  dialogs_prefix: String,
  translations_prefix: String,
  vfs: XrayVfs,
  files: Vec<DialogProjectFile>,
  findings: Vec<DialogFinding>,
}

impl DialogProject {
  /// Open a project over roots, reading every dialog file it exposes under the layout prefix.
  ///
  /// Two arguments, because opening answers two questions: roots say which trees are searched and
  /// in what order, and a layout says where inside them this domain keeps its data.
  ///
  /// A file that cannot be read becomes a finding and the project still opens: refusing the whole
  /// tree over one bad file would make the editor unable to reach the file you opened it to fix.
  ///
  /// # Errors
  ///
  /// Returns an error when the roots cannot be mounted, and a not-found error when it exposes no
  /// dialog files under the prefix. The second means the caller named the wrong place, and answering
  /// with an empty project would hide that.
  pub fn open(roots: &XrayRoots, layout: &DialogProjectLayout) -> XrfResult<Self> {
    Self::from_vfs(roots.open()?, roots, layout)
  }

  /// Open a project over roots somebody else mounted.
  ///
  /// The spec is still required, because it is what the descriptor echoes back so a follow-up read
  /// addresses the tree the open searched.
  ///
  /// # Errors
  ///
  /// Returns a not-found error when the roots exposes no dialog files under the dialogs prefix.
  pub fn from_vfs(vfs: XrayVfs, roots: &XrayRoots, layout: &DialogProjectLayout) -> XrfResult<Self> {
    let dialogs_prefix: String = layout.get_dialogs_prefix().to_owned();
    let scope: XrayLookupScope = XrayLookupScope::all().with_prefix(&dialogs_prefix)?;
    let assets: Vec<XrayAsset> = Self::list_dialog_assets(&vfs.scoped(&scope));

    if assets.is_empty() {
      return Err(XrfError::new_not_found_error(format!(
        "No dialog files under '{dialogs_prefix}' in {}",
        roots.describe()
      )));
    }

    let mut files: Vec<DialogProjectFile> = Vec::new();
    let mut findings: Vec<DialogFinding> = Vec::new();

    for asset in assets {
      let logical_path: String = asset.get_logical_path().as_str().to_owned();

      match Self::read_asset(&vfs.scoped(&scope), &logical_path) {
        Ok(file) => {
          for issue in file.get_issues() {
            findings.push(DialogFinding::new(
              "dialog.schema",
              Some(logical_path.clone()),
              issue.to_string(),
            ));
          }

          files.push(DialogProjectFile {
            logical_path,
            physical_path: asset.to_physical_path(),
            file,
          });
        }
        Err(error) => findings.push(DialogFinding::new(
          "dialog.unreadable",
          Some(logical_path),
          error.to_string(),
        )),
      }
    }

    Ok(Self {
      roots: roots.clone(),
      mode: layout.mode,
      dialogs_prefix,
      translations_prefix: layout.get_translations_prefix().to_owned(),
      vfs,
      files,
      findings,
    })
  }

  /// Every dialog asset a scoped roots exposes, in logical-path order.
  ///
  /// Sorted because mount order is not name order, and an index is only comparable across runs and
  /// machines if it depends on neither.
  pub fn list_dialog_assets(scoped: &XrayScopedVfs) -> Vec<XrayAsset> {
    let mut assets: Vec<XrayAsset> = scoped
      .list_entries()
      .into_iter()
      .filter(|asset| Self::is_dialog_logical_path(asset.get_logical_path()))
      .collect();

    assets.sort_by(|left, right| left.get_logical_path().as_str().cmp(right.get_logical_path().as_str()));

    assets
  }

  /// Whether a logical path names dialog data, by its file name.
  ///
  /// Takes the path type rather than a string so the last-component rule is the one `xrf-vfs` owns:
  /// a `\`-separated identity split with `std::path` answers the whole path on Linux.
  pub fn is_dialog_logical_path(logical_path: &XrayLogicalPath) -> bool {
    logical_path.has_extension(XML_SUFFIX) && logical_path.file_name().starts_with(DIALOG_FILE_PREFIX)
  }

  pub fn get_mode(&self) -> DialogProjectMode {
    self.mode
  }

  /// The roots this project was opened over, as the caller named them.
  pub fn get_roots(&self) -> &XrayRoots {
    &self.roots
  }

  pub fn get_dialogs_prefix(&self) -> &str {
    &self.dialogs_prefix
  }

  pub fn get_translations_prefix(&self) -> &str {
    &self.translations_prefix
  }

  /// The mounted roots, for a caller that needs to read something beside the dialogs.
  pub fn get_vfs(&self) -> &XrayVfs {
    &self.vfs
  }

  pub fn get_files(&self) -> &[DialogProjectFile] {
    &self.files
  }

  pub fn get_findings(&self) -> &[DialogFinding] {
    &self.findings
  }

  /// The file at a logical path.
  pub fn find_file(&self, logical_path: &str) -> Option<&DialogProjectFile> {
    self
      .files
      .iter()
      .find(|file| file.get_logical_path().eq_ignore_ascii_case(logical_path))
  }

  /// One dialog, addressed the way the project index lists it: by file, then by id.
  ///
  /// Both names are required rather than searching every file for the id, because ids are not unique
  /// across a tree — a mod overlaying a dialog keeps the original's id on purpose — and answering
  /// with whichever copy was read first would silently pick one.
  pub fn find_dialog(&self, logical_path: &str, id: &str) -> Option<&Dialog> {
    self.find_file(logical_path)?.get_file().find_dialog(id)
  }

  /// Describe one dialog with every phrase it declares.
  ///
  /// The counterpart to [`Self::describe`], which lists 502 dialogs as summaries: this is what a
  /// selection fetches. Answers `None` for a file or an id the project does not hold, leaving the
  /// caller to say which of the two was wrong.
  pub fn describe_dialog(&self, logical_path: &str, id: &str) -> Option<DialogDescriptor> {
    let file: &DialogProjectFile = self.find_file(logical_path)?;
    let dialog: &Dialog = file.get_file().find_dialog(id)?;

    // Keyed by the path the project holds, not the one the caller typed: lookup is case-insensitive,
    // and echoing the caller's spelling back would hand out a key that does not match the index.
    Some(DialogDescriptor::new(file.get_logical_path(), dialog))
  }

  /// Total dialogs across every file the project read.
  pub fn sum_dialogs(&self) -> usize {
    self.files.iter().map(|file| file.get_file().get_dialogs().len()).sum()
  }

  /// Whether every file the project holds could be written back.
  ///
  /// False as soon as one winner is archived, which is what stops an editing session that could only
  /// half succeed. `xrf-ltx` draws the same line between its rewrite and its read-only check.
  ///
  /// Also false for a project holding nothing, matching `TranslationProjectDescriptor`. `all` over an
  /// empty set is vacuously true, and a surface that enables saving on that offers a save which can do
  /// nothing. Opening already refuses an empty project, so this is unreachable today — stated anyway,
  /// because the two crates answering one question differently is how it stops being unreachable.
  pub fn is_editable(&self) -> bool {
    !self.files.is_empty() && self.files.iter().all(DialogProjectFile::is_editable)
  }

  /// The project as it crosses the wire: the index, not the phrases.
  pub fn describe(&self) -> DialogProjectDescriptor {
    let mut files: IndexMap<String, DialogFileDescriptor> = IndexMap::new();

    for entry in &self.files {
      files.insert(
        entry.get_logical_path().to_owned(),
        DialogFileDescriptor {
          physical_path: entry.get_physical_path().map(to_portable_path_string),
          is_editable: entry.is_editable(),
          encoding: String::from(entry.get_file().get_encoding().name()),
          dialogs: entry
            .get_file()
            .get_dialogs()
            .iter()
            .map(|dialog| DialogSummaryDescriptor {
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
      roots: self.roots.clone(),
      dialogs_prefix: self.dialogs_prefix.clone(),
      translations_prefix: self.translations_prefix.clone(),
      is_editable: self.is_editable(),
      files,
      findings: self.findings.clone(),
    }
  }

  fn read_asset(scoped: &XrayScopedVfs, logical_path: &str) -> XrfResult<DialogFile> {
    DialogFile::read_from_bytes(&scoped.read_bytes(logical_path)?)
  }
}
