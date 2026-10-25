use indexmap::IndexMap;
use xrf_error::{XrfError, XrfResult};
use xrf_extension::XrayExtension;
use xrf_translation::{read_gamedata_in, read_source_in};
use xrf_utils::to_portable_path_string;
use xrf_vfs::{XrayAsset, XrayLogicalPath, XrayRoots, XraySearch};

use crate::conversation::dialog::Dialog;
use crate::conversation::dialog_element::DialogElement;
use crate::dialog_file::DialogFile;
use crate::project::descriptor::dialog_descriptor::DialogDescriptor;
use crate::project::descriptor::dialog_file_descriptor::DialogFileDescriptor;
use crate::project::descriptor::dialog_project_descriptor::DialogProjectDescriptor;
use crate::project::descriptor::dialog_reference_descriptor::DialogReferenceDescriptor;
use crate::project::descriptor::dialog_summary_descriptor::DialogSummaryDescriptor;
use crate::project::dialog_finding::DialogFinding;
use crate::project::dialog_project_file::DialogProjectFile;
use crate::project::dialog_project_layout::DialogProjectLayout;
use crate::project::dialog_project_mode::DialogProjectMode;
use crate::project::text::dialog_text_index::DialogTextIndex;
use crate::project::text::dialog_text_language::DialogTextLanguage;

/// Filename prefix that marks a logical path as dialog data.
const DIALOG_FILE_PREFIX: &str = "dialog";

/// An open dialog project: mounted roots, and every dialog file under its dialogs prefix.
pub struct DialogProject {
  roots: XrayRoots,
  mode: DialogProjectMode,
  dialogs_prefix: String,
  translations_prefix: String,
  search: XraySearch,
  files: Vec<DialogProjectFile>,
  text: DialogTextIndex,
  findings: Vec<DialogFinding>,
}

impl DialogProject {
  /// Open a project over roots, reading every dialog file it exposes under the layout prefix.
  ///
  /// # Errors
  ///
  /// Returns an error when the roots cannot be mounted, and a not-found error when it exposes no
  /// dialog files under the prefix. The second means the caller named the wrong place, and answering
  /// with an empty project would hide that.
  pub fn open(roots: &XrayRoots, layout: &DialogProjectLayout) -> XrfResult<Self> {
    Self::from_search(roots.to_search_plan()?.open()?, roots, layout)
  }

  /// Open a project over roots somebody else mounted.
  ///
  /// # Errors
  ///
  /// Returns a not-found error when the roots exposes no dialog files under the dialogs prefix.
  pub fn from_search(search: XraySearch, roots: &XrayRoots, layout: &DialogProjectLayout) -> XrfResult<Self> {
    let dialogs_prefix: String = layout.get_dialogs_prefix().to_owned();
    let dialogs: XraySearch = search.clone().with_prefix(&dialogs_prefix)?;
    let assets: Vec<XrayAsset> = Self::list_dialog_assets(&dialogs);

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

      match Self::read_asset(&dialogs, &asset) {
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

    let translations_prefix: String = layout.get_translations_prefix().to_owned();
    let text: DialogTextIndex = if layout.is_text_skipped {
      DialogTextIndex::default()
    } else {
      Self::read_text(&search, roots, layout.mode, &translations_prefix, &mut findings)
    };

    Ok(Self {
      roots: roots.clone(),
      mode: layout.mode,
      dialogs_prefix,
      translations_prefix,
      search,
      files,
      text,
      findings,
    })
  }

  /// Read the text tree the dialogs resolve their lines from, over the roots already mounted.
  fn read_text(
    search: &XraySearch,
    roots: &XrayRoots,
    mode: DialogProjectMode,
    prefix: &str,
    findings: &mut Vec<DialogFinding>,
  ) -> DialogTextIndex {
    // The dialog layout decides the translation layout, because they are the same distinction seen
    // from two crates: XRF sources keep JSON beside the configs, shipped gamedata keeps per-language
    // XML under `configs\text`. Matched rather than converted — the two mode enums default opposite
    // ways on purpose, so a `From` between them that looked obvious would silently flip one.
    let read = match mode {
      DialogProjectMode::Source => read_source_in(search, roots, prefix),
      DialogProjectMode::Gamedata => read_gamedata_in(search, roots, prefix),
    };

    match read {
      Ok(descriptor) => {
        for finding in &descriptor.findings {
          findings.push(DialogFinding::new(
            format!("dialog.text.{}", finding.rule.trim_start_matches("translations.")),
            finding.subject.clone(),
            finding.message.clone(),
          ));
        }

        DialogTextIndex::from_descriptor(&descriptor)
      }
      Err(error) => {
        findings.push(DialogFinding::new(
          "dialog.text-unreadable",
          Some(prefix.to_owned()),
          format!("Dialog text could not be read, so phrases show their keys: {error}"),
        ));

        DialogTextIndex::default()
      }
    }
  }

  /// Every dialog asset a search reaches, in logical-path order.
  pub fn list_dialog_assets(search: &XraySearch) -> Vec<XrayAsset> {
    search
      .list_files()
      .into_iter()
      .filter(|asset| Self::is_dialog_logical_path(asset.get_logical_path()))
      .collect()
  }

  /// Whether a logical path names dialog data, by its file name.
  pub fn is_dialog_logical_path(logical_path: &XrayLogicalPath) -> bool {
    logical_path.has_extension(XrayExtension::Xml) && logical_path.file_name().starts_with(DIALOG_FILE_PREFIX)
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

  /// The search over the roots, for a caller that needs to read something beside the dialogs.
  pub fn get_search(&self) -> &XraySearch {
    &self.search
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

  /// One dialog, addressed the way the project index lists it: by file, then by id, since one id may be in many files.
  pub fn find_dialog(&self, logical_path: &str, id: &str) -> Option<&Dialog> {
    self.find_file(logical_path)?.get_file().find_dialog(id)
  }

  /// Every file declaring a dialog id, in logical-path order.
  pub fn list_files_declaring(&self, id: &str) -> Vec<&DialogProjectFile> {
    self
      .files
      .iter()
      .filter(|file| file.get_file().find_dialog(id).is_some())
      .collect()
  }

  /// The text tree this project resolves its phrase lines from.
  pub fn get_text(&self) -> &DialogTextIndex {
    &self.text
  }

  /// Languages the text tree offers, which is what a surface builds a language switcher from.
  pub fn list_languages(&self) -> &[String] {
    self.text.get_languages()
  }

  /// Describe one dialog with every phrase it declares, its lines in `language`, the tree's first where none is named;
  /// a language the tree does not hold resolves no line.
  pub fn describe_dialog(&self, logical_path: &str, id: &str, language: Option<&str>) -> Option<DialogDescriptor> {
    let file: &DialogProjectFile = self.find_file(logical_path)?;
    let dialog: &Dialog = file.get_file().find_dialog(id)?;
    let text: Option<DialogTextLanguage<'_>> = language
      .or_else(|| self.text.get_default_language())
      .and_then(|language| self.text.in_language(language));

    // Keyed by the path the project holds, not the one the caller typed: lookup is case-insensitive,
    // and echoing the caller's spelling back would hand out a key that does not match the index.
    Some(DialogDescriptor::new(file.get_logical_path(), dialog, text))
  }

  /// Every dialog and phrase element a predicate accepts, in file, dialog and document order.
  pub fn list_references(&self, accepts: impl Fn(&DialogElement) -> bool) -> Vec<DialogReferenceDescriptor> {
    let mut references: Vec<DialogReferenceDescriptor> = Vec::new();

    for file in &self.files {
      for dialog in file.get_file().get_dialogs() {
        let describe = |phrase_id: Option<&str>, element: &DialogElement, is_ignored: bool| DialogReferenceDescriptor {
          logical_path: file.get_logical_path().to_owned(),
          dialog_id: dialog.get_id().to_owned(),
          phrase_id: phrase_id.map(str::to_owned),
          element: element.into(),
          is_ignored,
        };

        for element in dialog.get_elements().iter().filter(|element| accepts(element)) {
          references.push(describe(None, element, false));
        }

        let is_entry_read: bool = dialog.is_entry_phrase_revisited();

        for phrase in dialog.get_phrases() {
          let is_entry: bool = dialog
            .get_entry_phrase()
            .is_some_and(|entry| std::ptr::eq(entry, phrase));

          for element in phrase.get_elements().iter().filter(|element| accepts(element)) {
            let is_ignored: bool = is_entry && !is_entry_read && element.get_kind().is_condition();

            references.push(describe(Some(phrase.get_id()), element, is_ignored));
          }
        }
      }
    }

    references
  }

  /// Total dialogs across every file the project read.
  pub fn sum_dialogs(&self) -> usize {
    self.files.iter().map(|file| file.get_file().get_dialogs().len()).sum()
  }

  /// Whether every file the project holds could be written back; a project of no files is not editable.
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
            .map(DialogSummaryDescriptor::from)
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
      languages: self.text.get_languages().to_vec(),
      text_keys: self.text.len(),
      files,
      findings: self.findings.clone(),
    }
  }

  fn read_asset(search: &XraySearch, asset: &XrayAsset) -> XrfResult<DialogFile> {
    DialogFile::read_from_bytes(&search.read_asset(asset)?)
  }
}
