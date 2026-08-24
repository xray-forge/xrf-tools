use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::Instant;

use xrf_error::{XrfError, XrfResult};
use xrf_shaders::{SHADER_SCRIPT_FILE_EXTENSION, ShaderRenderer, XRayShader, XRayShaderScript, is_shader_source_path};
use xrf_vfs::{XrayLookupScope, XrayVfs};

use crate::GamedataFindingFactory;
use crate::project::shaders::gamedata_shader_source_loader::GamedataShaderSourceLoader;
use crate::project::shaders::verify_shaders_result::GamedataShadersVerificationResult;
use crate::{GamedataCheckResult, GamedataProjectVerifyOptions, GamedataVerificationRule};

/// Logical directory holding the renderer shader trees.
pub(crate) const SHADERS_DIRECTORY: &str = "shaders";

pub(crate) struct ShadersVerifier<'a> {
  options: &'a GamedataProjectVerifyOptions,
  /// Logical root of the shader trees, not a filesystem path: sources resolve through the VFS.
  shaders_root: PathBuf,
  scope: &'a XrayLookupScope,
  vfs: &'a XrayVfs,
}

impl<'a> ShadersVerifier<'a> {
  pub(crate) fn new(vfs: &'a XrayVfs, scope: &'a XrayLookupScope, options: &'a GamedataProjectVerifyOptions) -> Self {
    Self {
      options,
      scope,
      shaders_root: PathBuf::from(SHADERS_DIRECTORY),
      vfs,
    }
  }

  pub(crate) fn verify(&self) -> GamedataShadersVerificationResult {
    xrf_output::heading!(self.options.output, "Verify renderer shaders:");

    let started_at: Instant = Instant::now();

    let mut result: GamedataShadersVerificationResult = GamedataShadersVerificationResult::default();

    for renderer in [ShaderRenderer::DirectX11, ShaderRenderer::OpenGl] {
      self.verify_renderer(renderer, &mut result);
    }

    result.sort_findings();

    result.duration = started_at.elapsed();

    xrf_output::info!(
      self.options.output,
      "Verified renderer shaders in {}, {}",
      xrf_utils::format_duration(result.duration),
      result.get_failure_message()
    );

    result
  }

  fn verify_renderer(&self, renderer: ShaderRenderer, result: &mut GamedataShadersVerificationResult) {
    xrf_output::verbose!(
      self.options.output,
      "Verify {} renderer shaders",
      renderer.display_name()
    );

    // Built with `\` rather than by joining host paths, so the prefix and every finding path read the same on Linux as on
    // Windows. `xrf-shaders` wants a `Path`, but the value inside it is an engine identity.
    let renderer_prefix: String = format!("{SHADERS_DIRECTORY}\\{}", renderer.directory_name());
    let renderer_root: PathBuf = PathBuf::from(&renderer_prefix);

    let entries: Vec<String> = match self.renderer_entries(&renderer_prefix) {
      Ok(entries) => entries,
      Err(error) => {
        result.add_finding(GamedataFindingFactory::for_asset(
          GamedataVerificationRule::ShadersSourceRead,
          &renderer_root,
          format!("Failed to enumerate renderer shaders: {error}"),
        ));

        return;
      }
    };

    // A VFS has no empty directories — they are derived from entries — so "nothing resolves here" is the only detectable
    // form of an absent renderer root, and it covers both a missing tree and an empty one.
    if entries.is_empty() {
      // Specific to OpenXray
      if renderer == ShaderRenderer::OpenGl {
        xrf_output::verbose!(
          self.options.output,
          "Skip OpenGL renderer shaders, the renderer root is not present"
        );

        return;
      }

      result.add_finding(GamedataFindingFactory::for_asset(
        GamedataVerificationRule::ShadersRendererRoot,
        &renderer_root,
        format!("{} renderer shader root is missing", renderer.display_name()),
      ));

      return;
    }

    self.verify_root_shader_scripts(&renderer_root, &renderer_prefix, result);

    let source_loader: GamedataShaderSourceLoader<'_> = GamedataShaderSourceLoader::new(self.vfs, self.scope);
    let mut checked_sources: HashSet<PathBuf> = HashSet::new();

    // Scripts and sources are different sets: a `.s` script is Lua the renderer runs, so a renderer holding only scripts is
    // still checked rather than reported absent.
    for source in entries.iter().filter(|path| is_shader_source_path(Path::new(path))) {
      self.verify_shader_source(
        Path::new(source),
        renderer,
        result,
        &source_loader,
        &mut checked_sources,
      );
    }
  }

  /// Every entry under one renderer root, whether loose or inside a volume.
  ///
  /// Sorted so a run reports in a stable order, which a single directory walk gave for free and enumeration across mounts does not.
  fn renderer_entries(&self, renderer_prefix: &str) -> XrfResult<Vec<String>> {
    let scope: XrayLookupScope = self.scope.clone().with_prefix(renderer_prefix)?;
    let mut entries: Vec<String> = self
      .vfs
      .scoped(&scope)
      .list_entries()
      .into_iter()
      .map(|location| location.get_logical_path().to_string())
      .collect();

    entries.sort();

    Ok(entries)
  }

  fn verify_root_shader_scripts(
    &self,
    renderer_root: &Path,
    renderer_prefix: &str,
    result: &mut GamedataShadersVerificationResult,
  ) {
    // Scripts sit directly in the renderer root, so this is a directory listing rather than a walk.
    let listing = match self.vfs.scoped(self.scope).list_children(renderer_prefix) {
      Ok(listing) => listing,
      Err(error) => {
        result.add_finding(GamedataFindingFactory::for_asset(
          GamedataVerificationRule::ShadersSourceRead,
          renderer_root,
          format!("Failed to read renderer shader root: {error}"),
        ));

        return;
      }
    };

    for file in listing.files {
      if !file
        .get_logical_path()
        .has_extension(&format!(".{SHADER_SCRIPT_FILE_EXTENSION}"))
      {
        continue;
      }

      // `xrf-shaders` keys sources by path, and for this project those keys are engine identities.
      let path: PathBuf = PathBuf::from(file.get_logical_path().as_str());

      result.increment_checked_scripts_count();

      match self.read_script(file.get_logical_path().as_str()) {
        Ok(source) => {
          if let Err(error) = XRayShaderScript::parse(&path, &source) {
            result.add_finding(GamedataFindingFactory::for_asset(
              GamedataVerificationRule::ShadersLuaSyntax,
              &path,
              error.to_string(),
            ));
          }
        }
        Err(error) => result.add_finding(GamedataFindingFactory::for_asset(
          GamedataVerificationRule::ShadersSourceRead,
          &path,
          format!("Failed to read shader script: {error}"),
        )),
      }
    }
  }

  /// Reads one shader script as text, through the same mounts its sources come from.
  fn read_script(&self, logical_path: &str) -> Result<String, XrfError> {
    let bytes: Vec<u8> = self.vfs.scoped(self.scope).read_bytes(logical_path)?;

    String::from_utf8(bytes).map_err(|error| XrfError::new_read_error(format!("not valid utf-8: {error}")))
  }

  fn verify_shader_source(
    &self,
    path: &Path,
    renderer: ShaderRenderer,
    result: &mut GamedataShadersVerificationResult,
    source_loader: &GamedataShaderSourceLoader,
    checked_sources: &mut HashSet<PathBuf>,
  ) {
    if checked_sources.contains(path) {
      return;
    }

    match XRayShader::load(path, renderer, &self.shaders_root, source_loader) {
      Ok(shader) => Self::record_checked_shader_sources(&shader, checked_sources, result),
      Err(error) => {
        checked_sources.insert(path.to_path_buf());
        result.increment_checked_sources_count();
        result.add_finding(GamedataFindingFactory::for_asset(
          Self::shader_error_rule_id(&error),
          path,
          error.to_string(),
        ));
      }
    }
  }

  fn has_extension(path: &Path, extension: &str) -> bool {
    path
      .extension()
      .and_then(|value| value.to_str())
      .is_some_and(|value| value.eq_ignore_ascii_case(extension))
  }

  fn shader_error_rule_id(error: &XrfError) -> GamedataVerificationRule {
    match error {
      XrfError::Invalid { .. } => GamedataVerificationRule::ShadersIncludeSyntax,
      XrfError::NotFound { .. } => GamedataVerificationRule::ShadersIncludeMissing,
      XrfError::Read { .. } | XrfError::Io { .. } => GamedataVerificationRule::ShadersSourceRead,
      XrfError::Verify { .. } => GamedataVerificationRule::ShadersIncludeCycle,
      _ => GamedataVerificationRule::ShadersSourceInvalid,
    }
  }

  fn record_checked_shader_sources(
    shader: &XRayShader,
    checked_sources: &mut HashSet<PathBuf>,
    result: &mut GamedataShadersVerificationResult,
  ) {
    if !checked_sources.insert(shader.path().to_path_buf()) {
      return;
    }

    result.increment_checked_sources_count();

    for import in shader.imports() {
      Self::record_checked_shader_sources(import.shader(), checked_sources, result);
    }
  }
}

#[cfg(test)]
mod tests {
  use std::fs;
  use std::path::{Path, PathBuf};

  use xrf_error::{XrfError, XrfResult};
  use xrf_vfs::{XrayLookupScope, XrayVfs};

  use super::{SHADERS_DIRECTORY, ShadersVerifier};
  use crate::{GamedataCheckResult, GamedataProjectVerifyOptions, GamedataVerificationRule};

  /// Mounts a gamedata tree, since the verifier addresses shaders by engine identity rather than by filesystem path.
  ///
  /// A directory mount rather than a packed volume: what these check is enumeration and include resolution over logical
  /// paths, which a directory source exercises identically while staying fast.
  fn mount(root: &Path) -> XrfResult<(XrayVfs, XrayLookupScope)> {
    let mut vfs: XrayVfs = XrayVfs::new();

    vfs.mount_directory("", root)?;

    Ok((vfs, XrayLookupScope::all()))
  }

  #[test]
  fn validates_d3d11_scripts_and_renderer_then_root_includes() -> XrfResult {
    let root: PathBuf = create_gamedata_root("d3d11")?;
    let shaders: PathBuf = root.join(SHADERS_DIRECTORY);
    let options: GamedataProjectVerifyOptions = GamedataProjectVerifyOptions {
      output: xrf_output::OutputOptions::default(),
      ..Default::default()
    };

    write_file(
      &shaders.join("r3/basic.s"),
      "function normal(shader, t_base, t_second, t_detail) end\n",
    )?;
    write_file(&shaders.join("r3/main.ps"), "#include \"shared/common.h\"\n")?;
    write_file(&shaders.join("shared/common.h"), "float value;\n")?;

    let (vfs, scope) = mount(&root)?;
    let result = ShadersVerifier::new(&vfs, &scope, &options).verify();

    assert_eq!(
      result.get_failure_message(),
      "1 shader scripts and 2 shader sources checked, 0 problems"
    );
    assert!(result.get_findings().is_empty());

    fs::remove_dir_all(root)?;

    Ok(())
  }

  #[test]
  fn skips_a_missing_open_gl_renderer_root() -> XrfResult {
    let root: PathBuf = create_gamedata_root("missing-open-gl")?;
    let options: GamedataProjectVerifyOptions = GamedataProjectVerifyOptions {
      output: xrf_output::OutputOptions::default(),
      ..Default::default()
    };

    fs::remove_dir(root.join(SHADERS_DIRECTORY).join("gl"))?;
    write_file(
      &root.join(SHADERS_DIRECTORY).join("r3/basic.s"),
      "function normal(shader, t_base, t_second, t_detail) end\n",
    )?;

    let (vfs, scope) = mount(&root)?;
    let result = ShadersVerifier::new(&vfs, &scope, &options).verify();

    assert!(result.get_findings().is_empty());
    assert_eq!(
      result.get_failure_message(),
      "1 shader scripts and 0 shader sources checked, 0 problems",
      "a renderer holding only scripts is still checked, not reported absent"
    );

    fs::remove_dir_all(root)?;

    Ok(())
  }

  #[test]
  fn reports_lua_include_and_cycle_problems_together() -> XrfResult {
    let root: PathBuf = create_gamedata_root("static-problems")?;
    let shaders: PathBuf = root.join(SHADERS_DIRECTORY);
    let options: GamedataProjectVerifyOptions = GamedataProjectVerifyOptions {
      output: xrf_output::OutputOptions::default(),
      ..Default::default()
    };

    write_file(&shaders.join("r3/basic.s"), "function normal(s) end\n")?;
    write_file(&shaders.join("gl/invalid.s"), "function normal(\n")?;
    write_file(&shaders.join("gl/missing.ps"), "#include \"missing.h\"\n")?;
    write_file(&shaders.join("gl/first.h"), "#include \"second.h\"\n")?;
    write_file(&shaders.join("gl/second.h"), "#include \"first.h\"\n")?;

    let (vfs, scope) = mount(&root)?;
    let result = ShadersVerifier::new(&vfs, &scope, &options).verify();
    let rule_ids: Vec<String> = result
      .get_findings()
      .iter()
      .map(|finding| finding.rule_id().to_string())
      .collect();

    assert!(rule_ids.contains(&GamedataVerificationRule::ShadersLuaSyntax.to_string()));
    assert!(rule_ids.contains(&GamedataVerificationRule::ShadersIncludeMissing.to_string()));
    assert!(rule_ids.contains(&GamedataVerificationRule::ShadersIncludeCycle.to_string()));

    fs::remove_dir_all(root)?;

    Ok(())
  }

  /// Creates a gamedata root holding an empty `shaders` tree, returning the gamedata root.
  ///
  /// The verifier looks under the logical `shaders` directory, so the tree has to sit where a real one does rather than
  /// being the root itself.
  fn create_gamedata_root(test_name: &str) -> XrfResult<PathBuf> {
    let root: PathBuf = std::env::temp_dir().join("xrf-gamedata-shader-tests").join(test_name);

    if root.exists() {
      fs::remove_dir_all(&root)?;
    }

    fs::create_dir_all(root.join(SHADERS_DIRECTORY).join("r3"))?;
    fs::create_dir_all(root.join(SHADERS_DIRECTORY).join("gl"))?;

    Ok(root)
  }

  fn write_file(path: &Path, contents: &str) -> XrfResult {
    let parent: &Path = path
      .parent()
      .ok_or_else(|| XrfError::new_unexpected_error(format!("Shader test path has no parent: {}", path.display())))?;

    fs::create_dir_all(parent)?;
    fs::write(path, contents)?;

    Ok(())
  }
}
