use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::time::SystemTime;

use xrf_error::{XrfError, XrfResult};

use crate::shader::shader_composer::ShaderComposer;

/// Every WGSL module the renderer has, by the name an `#import` gives it: its path under `shaders/`, without the
/// extension, as the build script lists them.
const EMBEDDED: &[(&str, &str)] = include!(concat!(env!("OUT_DIR"), "/shader_modules.rs"));

/// Modules that need a device feature, each with the module a device without it composes in its place: the same
/// functions over nothing, so a pass importing one draws on every device and does more where the feature is.
const FALLBACKS: &[(&str, wgpu::Features, &str)] = &[(
  "common/scene_trace",
  wgpu::Features::EXPERIMENTAL_RAY_QUERY,
  "common/scene_trace_none",
)];

/// The renderer's WGSL modules: embedded in a release build, read from the crate's `shaders/` directory in a debug
/// one so an edited file is drawn with on the next reload.
#[derive(Clone, Debug)]
pub struct ShaderLibrary {
  modules: BTreeMap<&'static str, String>,
  /// When each module's file last changed, for a debug build's reload.
  stamps: BTreeMap<&'static str, Option<SystemTime>>,
}

impl Default for ShaderLibrary {
  fn default() -> Self {
    let mut library: Self = Self {
      modules: EMBEDDED
        .iter()
        .map(|(name, source)| (*name, source.to_string()))
        .collect(),
      stamps: BTreeMap::new(),
    };

    if cfg!(debug_assertions) {
      library.reload();
    }

    library
  }
}

impl ShaderLibrary {
  /// The names of every module.
  pub fn list_modules(&self) -> impl Iterator<Item = &'static str> + '_ {
    self.modules.keys().copied()
  }

  /// A module with its imports inlined, every feature taken as there.
  ///
  /// # Errors
  ///
  /// Returns an error for an unknown module or import.
  pub fn compose(&self, entry: &str) -> XrfResult<String> {
    self.compose_for(entry, wgpu::Features::all())
  }

  /// A module with its imports inlined for a device with `features`: an import needing one it lacks composed as its
  /// fallback.
  ///
  /// # Errors
  ///
  /// Returns an error for an unknown module or import.
  pub fn compose_for(&self, entry: &str, features: wgpu::Features) -> XrfResult<String> {
    ShaderComposer::compose(entry, &|name: &str| {
      let name: &str = FALLBACKS
        .iter()
        .find(|(module, needed, _)| *module == name && !features.contains(*needed))
        .map_or(name, |(_, _, fallback)| fallback);

      self
        .modules
        .get(name)
        .map(String::as_str)
        .ok_or_else(|| XrfError::new_not_found_error(format!("No shader module '{name}'")))
    })
  }

  /// Where a debug build reads the modules' sources from: the crate's own `shaders` folder.
  pub fn get_source_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("shaders")
  }

  /// The modules that compose from any of `changed`, themselves included: those whose pipelines a reload of them
  /// stales. A changed fallback stands for the module it replaces too; a module that no longer composes is taken as
  /// stale, so its pipelines are asked again and report why.
  pub fn list_dependents(&self, changed: &[&str]) -> Vec<&'static str> {
    let changed: BTreeSet<&str> = changed
      .iter()
      .copied()
      .chain(
        FALLBACKS
          .iter()
          .filter(|(_, _, fallback)| changed.contains(fallback))
          .map(|(module, _, _)| *module),
      )
      .collect();
    let read = |module: &str| {
      self
        .modules
        .get(module)
        .map(String::as_str)
        .ok_or_else(|| XrfError::new_not_found_error(format!("No shader module '{module}'")))
    };

    self
      .modules
      .keys()
      .copied()
      .filter(|name| {
        ShaderComposer::list_modules(name, &read)
          .map_or(true, |used| used.iter().any(|module| changed.contains(module.as_str())))
      })
      .collect()
  }

  /// Reads again every module whose file changed since it was last read, in a debug build; answers which did.
  pub fn reload(&mut self) -> Vec<&'static str> {
    if !cfg!(debug_assertions) {
      return Vec::new();
    }

    let root: PathBuf = Self::get_source_root();
    let mut changed: Vec<&'static str> = Vec::new();

    for (name, _) in EMBEDDED {
      let path: PathBuf = root.join(format!("{name}.wgsl"));
      let stamp: Option<SystemTime> = std::fs::metadata(&path).and_then(|it| it.modified()).ok();

      if self.stamps.get(name) == Some(&stamp) {
        continue;
      }

      // A file an editor holds mid-save is read again next time, its stamp not yet taken.
      let Ok(source) = std::fs::read_to_string(&path) else {
        continue;
      };

      self.stamps.insert(name, stamp);

      if self.modules.get(name) != Some(&source) {
        log::info!("Shader module '{name}' changed on disk");
        self.modules.insert(name, source);
        changed.push(name);
      }
    }

    changed
  }
}
