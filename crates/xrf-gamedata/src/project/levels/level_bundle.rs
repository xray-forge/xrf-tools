use xrf_job::JobHandle;
use xrf_vfs::XrayAssetType;

use crate::GamedataProject;
use crate::project::levels::level_engine_constants::LEVELS_DIRECTORY;

/// One level bundle directory, addressed the way the engine addresses `$level$`.
///
/// Rules receive a bundle rather than a name so that finding subjects, on-disk lookups and
/// level-local asset resolution all derive from one place.
pub(crate) struct LevelBundle<'a> {
  job: &'a JobHandle,
  project: &'a GamedataProject,
  name: &'a str,
}

impl<'a> LevelBundle<'a> {
  pub(crate) fn new(project: &'a GamedataProject, name: &'a str, job: &'a JobHandle) -> Self {
    Self { project, name, job }
  }

  pub(crate) fn job(&self) -> &JobHandle {
    self.job
  }

  pub(crate) fn name(&self) -> &str {
    self.name
  }

  pub(crate) fn project(&self) -> &'a GamedataProject {
    self.project
  }

  /// Logical path of the bundle directory, used as a finding subject.
  pub(crate) fn path(&self) -> String {
    Self::path_of(self.name)
  }

  /// Logical path of a file inside the bundle, used as a finding subject.
  pub(crate) fn file_path(&self, file: &str) -> String {
    format!("{LEVELS_DIRECTORY}\\{}\\{file}", self.name)
  }

  /// Logical path of a bundle directory that does not need to exist.
  pub(crate) fn path_of(name: &str) -> String {
    format!("{LEVELS_DIRECTORY}\\{name}")
  }

  /// Logical path of a bundle file the project actually holds, loose or archived.
  pub(crate) fn resolved_file(&self, file: &str) -> Option<String> {
    let logical: String = self.file_path(file);

    self
      .project
      .vfs()
      .scoped(self.project.scope())
      .find(&logical)
      .ok()
      .flatten()
      .map(|_| logical)
  }

  /// Size of a bundle file without reading it, so a truncated asset is caught before parsing.
  pub(crate) fn file_size(&self, file: &str) -> Option<u64> {
    self.project.size(&self.file_path(file))
  }

  pub(crate) fn contains(&self, file: &str) -> bool {
    self.file_size(file).is_some()
  }

  /// Resolve a texture reference the way the renderer does.
  ///
  /// `CRender::texture_load` probes `$level$` before `$game_textures$`, so lightmaps and terrain
  /// atlases shipped inside the bundle resolve even though they are absent from the shared texture
  /// tree. `$game_saves$` is probed in between, but it is not part of gamedata.
  pub(crate) fn resolves_texture(&self, reference: &str) -> bool {
    let logical: String = XrayAssetType::Dds
      .get_rules()
      .expect("dds has rules")
      .to_logical_path(reference);

    let in_bundle: bool = self
      .project
      .vfs()
      .scoped(self.project.scope())
      .find(&format!("{}\\{logical}", self.path()))
      .ok()
      .flatten()
      .is_some();

    in_bundle
      || self
        .project
        .vfs()
        .scoped(self.project.scope())
        .dds_texture(reference)
        .ok()
        .flatten()
        .is_some()
  }
}
