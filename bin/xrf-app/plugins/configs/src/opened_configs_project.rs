use std::sync::Arc;

use xrf_app_core::assets::AssetMountState;
use xrf_app_core::session::{SessionDerived, SessionId, SessionSnapshot};
use xrf_app_core::types::TauriResult;
use xrf_dltx::select_ltx_dialect;
use xrf_ltx::{LtxDocumentSource, LtxProject, LtxProjectOptions};
use xrf_ltx_inspect::{LtxInventory, LtxInventoryReader};
use xrf_utils::{error_to_string, format_path};
use xrf_vfs::{XrayInputRecord, XrayRevision, XrayRoots, XraySearch};

use crate::configs_project::ConfigsProject;
use crate::descriptor::ConfigsProjectDescriptor;

/// A configs project assembled over the shared asset roots and listed, with what it read: what an open commits, and
/// what a commit's follow reads again.
pub(crate) struct OpenedConfigsProject {
  project: LtxProject,
  inputs: XrayInputRecord,
  /// The project revision of the snapshot it was read from.
  revision: XrayRevision,
  inventory: LtxInventory,
  roots: XrayRoots,
  prefix: Option<String>,
  is_dltx: bool,
}

impl OpenedConfigsProject {
  /// Assembles the project over `search`, recording what it reads, and lists what it holds.
  ///
  /// # Errors
  ///
  /// Returns an error when the project cannot be assembled, or a config it holds cannot be read.
  pub fn read(search: &XraySearch, roots: XrayRoots, prefix: Option<String>, is_dltx: bool) -> TauriResult<Self> {
    let inputs: XrayInputRecord = XrayInputRecord::new();
    let options: LtxProjectOptions = LtxProjectOptions {
      dialect: select_ltx_dialect(is_dltx),
      is_caching_resolutions: true,
      is_with_schemes_check: true,
      is_strict_check: false,
    };
    let project: LtxProject = LtxProject::open_at_search_opt(
      roots.describe(),
      &search.clone().with_recording(&inputs),
      prefix.as_deref(),
      options,
    )
    .map_err(error_to_string)?;
    let inventory: LtxInventory = {
      let source = project.document_source();

      LtxInventoryReader::new(&project, &source as &dyn LtxDocumentSource)
        .read()
        .map_err(error_to_string)?
    };

    Ok(Self {
      project,
      inputs,
      revision: search.get_vfs().get_revision(),
      inventory,
      roots,
      prefix,
      is_dltx,
    })
  }

  /// The open project read again over the roots it was opened over, once a commit moved a config it read.
  ///
  /// # Errors
  ///
  /// As [`Self::read`], or when the roots cannot be searched.
  pub fn derive(
    current: &SessionSnapshot<ConfigsProject>,
    assets: &AssetMountState,
  ) -> TauriResult<SessionDerived<ConfigsProject>> {
    let descriptor: &ConfigsProjectDescriptor = &current.descriptor;
    let (project, inputs) = Self::read(
      &assets.search(&descriptor.roots)?,
      descriptor.roots.clone(),
      descriptor.prefix.clone(),
      descriptor.is_dltx,
    )?
    .into_project(current.session_id);

    Ok(SessionDerived::new(project, inputs))
  }

  pub fn get_revision(&self) -> XrayRevision {
    self.revision
  }

  /// The session value the project is opened as under `session_id`, with what it was read from.
  pub fn into_project(self, session_id: SessionId) -> (ConfigsProject, XrayInputRecord) {
    let descriptor: Arc<ConfigsProjectDescriptor> = Arc::new(ConfigsProjectDescriptor {
      declared_schemes: self
        .project
        .ltx_scheme_declarations
        .keys()
        .map(String::from)
        .collect::<Vec<String>>(),
      inventory: self.inventory,
      is_dltx: self.is_dltx,
      prefix: self.prefix,
      root: format_path(&self.project.root).to_string(),
      roots: self.roots,
      session_id,
    });

    (ConfigsProject::new(self.project, descriptor), self.inputs)
  }
}
