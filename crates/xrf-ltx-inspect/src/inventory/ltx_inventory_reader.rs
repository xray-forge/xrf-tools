use std::collections::{HashMap, HashSet};

use xrf_error::XrfResult;
use xrf_ltx::{Ltx, LtxDocument, LtxDocumentSource, LtxProject};
use xrf_vfs::{XrayAsset, XrayLogicalPath};

use crate::inventory::{LtxInventory, LtxInventoryFile, LtxInventoryRole};

/// Which configs name each config in an `#include`, keyed by the named one.
type Includers = HashMap<String, Vec<String>>;

/// Reads a project as the inventory a tree and a listing are both drawn from.
pub struct LtxInventoryReader<'a> {
  project: &'a LtxProject,
  source: &'a dyn LtxDocumentSource,
}

impl<'a> LtxInventoryReader<'a> {
  /// A reader over one open project and the world its configs came from.
  pub fn new(project: &'a LtxProject, source: &'a dyn LtxDocumentSource) -> Self {
    Self { project, source }
  }

  /// Every config the project holds, with its role and the mount that supplied it.
  ///
  /// # Errors
  ///
  /// Returns an error when the dialect cannot list what patches what, when an include cannot be resolved, or when the
  /// mounted world cannot be searched.
  pub fn read(&self) -> XrfResult<LtxInventory> {
    let attachments: HashSet<String> = self.list_attachments()?;
    let entries: HashSet<&str> = self.project.ltx_file_entries.iter().map(|it| it.as_str()).collect();
    let scheme_files: HashSet<&str> = self.project.ltx_scheme_files.iter().map(|it| it.as_str()).collect();
    let mut includers: Includers = self.list_includers()?;

    let mut files: Vec<LtxInventoryFile> = Vec::with_capacity(self.project.ltx_files.len());

    for path in &self.project.ltx_files {
      let asset: Option<XrayAsset> = self.project.vfs().scoped(self.project.scope()).find(path.as_str())?;

      files.push(LtxInventoryFile {
        is_physical: asset.as_ref().is_some_and(|it| it.to_physical_path().is_some()),
        role: Self::role_of(path, &entries, &scheme_files, &attachments, &mut includers),
        source: asset
          .as_ref()
          .map_or_else(|| self.project.root.display().to_string(), XrayAsset::format_container),
        path: String::from(path.as_str()),
      });
    }

    Ok(LtxInventory { files })
  }

  /// What one config is to the project.
  ///
  /// Ordered by which fact a reader most needs: a scheme declaration is never verified against schemes, an attachment
  /// never stands on its own, and only then does "does anything include it" decide.
  fn role_of(
    path: &XrayLogicalPath,
    entries: &HashSet<&str>,
    scheme_files: &HashSet<&str>,
    attachments: &HashSet<String>,
    includers: &mut Includers,
  ) -> LtxInventoryRole {
    if scheme_files.contains(path.as_str()) {
      return LtxInventoryRole::SchemeFile;
    }

    if attachments.contains(path.as_str()) {
      return LtxInventoryRole::Attachment;
    }

    if entries.contains(path.as_str()) {
      return LtxInventoryRole::EntryPoint;
    }

    LtxInventoryRole::Included {
      by: includers.remove(path.as_str()).unwrap_or_default(),
    }
  }

  /// Configs the dialect says patch another rather than standing on their own.
  ///
  /// Standard LTX answers nothing here, so this is the whole cost of supporting a patch dialect from a crate that does
  /// not know one exists.
  fn list_attachments(&self) -> XrfResult<HashSet<String>> {
    let roots: Vec<String> = self
      .project
      .ltx_files
      .iter()
      .map(|path| String::from(path.as_str()))
      .collect();

    Ok(
      self
        .project
        .get_dialect()
        .plan_attachments(&roots, self.source)?
        .into_iter()
        .collect(),
    )
  }

  /// The include graph, reversed: for each config, the configs whose `#include` reaches it.
  ///
  /// A config the project does not hold is dropped rather than recorded, so an edge always names a file the tree can
  /// show. An unreadable config contributes no edges: its own statements are unknown, and the project already treats
  /// it as an entry point for exactly that reason.
  fn list_includers(&self) -> XrfResult<Includers> {
    let held: HashSet<&str> = self.project.ltx_files.iter().map(|it| it.as_str()).collect();
    let mut includers: Includers = HashMap::new();

    for path in &self.project.ltx_files {
      let Ok(Some(document)) = self.source.read_document(path.as_str()) else {
        continue;
      };

      for included in Self::list_included(&document, path.as_str(), self.source)? {
        if !held.contains(included.as_str()) {
          continue;
        }

        includers.entry(included).or_default().push(String::from(path.as_str()));
      }
    }

    Ok(includers)
  }

  /// Every config one document's `#include` statements name, expanded through the source.
  fn list_included(document: &LtxDocument, path: &str, source: &dyn LtxDocumentSource) -> XrfResult<Vec<String>> {
    let directory: &str = Ltx::directory_of(path);
    let mut included: Vec<String> = Vec::new();

    for statement in document.list_included() {
      included.extend(source.resolve_include(directory, statement)?);
    }

    Ok(included)
  }
}
