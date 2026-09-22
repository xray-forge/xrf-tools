use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use rayon::prelude::*;
use xrf_error::{XrfError, XrfResult};
use xrf_extension::XrayExtension;
use xrf_vfs::{XrayCachePolicy, XrayLogicalPath, XrayLookupScope, XrayRoots, XrayVfs};

use crate::dialect::{LtxDialect, LtxResolution, LtxResolveRequest, LtxStandardDialect};
use crate::document::LtxDocument;
use crate::ltx::{Ltx, LtxSectionSchemes};
use crate::project::{LtxProjectOptions, LtxReadCounters, LtxReadCountersSnapshot, LtxResolvedRoot};
use crate::scheme::LtxSchemeParser;
use crate::source::{LtxDocumentSource, LtxIncludeSource, LtxListingCache, LtxVfsSource};
use crate::syntax::{LTX_SCHEME_EXTENSION, LTX_SCHEME_LTX_FILENAME, SYSTEM_LTX_FILENAME};

/// An LTX project over one VFS scope. Files use logical paths for both loose and archived configs.
#[derive(Debug)]
pub struct LtxProject {
  /// Location shown in project output, on the host filesystem.
  pub root: PathBuf,
  /// LTX entry points not included by another config in scope.
  pub ltx_file_entries: Vec<XrayLogicalPath>,
  /// Every LTX logical path in scope.
  pub ltx_files: Vec<XrayLogicalPath>,
  /// Scheme-definition LTX paths in scope.
  pub ltx_scheme_files: Vec<XrayLogicalPath>,
  /// Scheme entry points not included by another config in scope.
  pub ltx_scheme_file_entries: Vec<XrayLogicalPath>,
  /// Section schemes declared by scheme entry points.
  pub ltx_scheme_declarations: LtxSectionSchemes,
  /// Configs the dialect says patch another rather than standing alone, planned once while assembling.
  pub ltx_attachments: Vec<String>,
  /// Mounted sources that resolve project files.
  vfs: XrayVfs,
  scope: XrayLookupScope,
  /// How much reading and parsing this project has done.
  counters: Arc<LtxReadCounters>,
  /// Which rules resolve this project's configs.
  dialect: Arc<dyn LtxDialect>,
  /// Whether a resolved root is kept for the life of the project, which only a caller that reopens roots wants.
  is_caching_resolutions: bool,
  /// Roots resolved so far, one cell per root so two threads asking at once produce one resolution between them.
  resolved: Mutex<HashMap<XrayLogicalPath, Arc<LtxResolvedRoot>>>,
  /// Directory listings every source of this project shares, since DLTX asks for one per root.
  listings: LtxListingCache,
}

impl LtxProject {
  /// Opens a project over one directory mounted at the logical root.
  ///
  /// # Errors
  ///
  /// Returns an error when the directory cannot be indexed or its project files cannot be assembled.
  pub fn open_at_path_opt<P: AsRef<Path>>(root: P, options: LtxProjectOptions) -> XrfResult<Self> {
    let root: &Path = root.as_ref();
    let mut vfs: XrayVfs = XrayVfs::new().with_cache_policy(XrayCachePolicy::configs());

    vfs.mount_directory("", root)?;

    Self::assemble(root.to_path_buf(), vfs, XrayLookupScope::all(), options)
  }

  /// Opens a directory-backed project with default options.
  pub fn open_at_path<P: AsRef<Path>>(root: P) -> XrfResult<Self> {
    Self::open_at_path_opt(root, Default::default())
  }

  /// Opens a project over roots.
  ///
  /// # Errors
  ///
  /// Returns an error when the roots cannot be mounted, the prefix is not a logical path, or the project cannot be
  /// assembled.
  pub fn open_at_roots_opt(roots: &XrayRoots, prefix: Option<&str>, options: LtxProjectOptions) -> XrfResult<Self> {
    let vfs: XrayVfs = roots.open()?.with_cache_policy(XrayCachePolicy::configs());
    let scope: XrayLookupScope = match prefix {
      Some(prefix) => XrayLookupScope::all().with_prefix(prefix)?,
      None => XrayLookupScope::all(),
    };

    Self::open_at_scope_opt(roots.describe(), vfs, scope, options)
  }

  /// Opens a project from an existing VFS scope.
  ///
  /// `root` is reported in user-facing output; the mounts and scope determine which files the project can read.
  ///
  /// # Errors
  ///
  /// Returns an error when a config cannot be read, an include cannot be resolved, or a scheme declaration is invalid.
  pub fn open_at_scope_opt(
    root: impl AsRef<Path>,
    vfs: XrayVfs,
    scope: XrayLookupScope,
    options: LtxProjectOptions,
  ) -> XrfResult<Self> {
    Self::assemble(root.as_ref().to_path_buf(), vfs, scope, options)
  }

  /// Creates an empty project for callers that need the project shape without mounted files.
  pub fn empty(root: impl AsRef<Path>) -> Self {
    Self {
      counters: LtxReadCounters::new_shared(),
      dialect: Arc::new(LtxStandardDialect),
      is_caching_resolutions: false,
      resolved: Mutex::default(),
      listings: LtxListingCache::default(),
      ltx_file_entries: Vec::new(),
      ltx_files: Vec::new(),
      ltx_attachments: Vec::new(),
      ltx_scheme_declarations: Default::default(),
      ltx_scheme_file_entries: Vec::new(),
      ltx_scheme_files: Vec::new(),
      root: root.as_ref().to_path_buf(),
      scope: XrayLookupScope::all(),
      vfs: XrayVfs::new(),
    }
  }

  /// Collects files, identifies entry points from includes, and parses scheme declarations.
  fn assemble(root: PathBuf, vfs: XrayVfs, scope: XrayLookupScope, options: LtxProjectOptions) -> XrfResult<Self> {
    let counters: Arc<LtxReadCounters> = LtxReadCounters::new_shared();
    let listings: LtxListingCache = LtxListingCache::default();
    let source: LtxVfsSource = LtxVfsSource::new_counted(&vfs, &scope, &counters).with_listings(&listings);

    let ltx_files: Vec<XrayLogicalPath> = Self::collect_logical_paths(&vfs, &scope)?;

    // A name test over the listing, which needs no reading at all and therefore no place in the walk below.
    let ltx_scheme_files: Vec<XrayLogicalPath> = if options.is_with_schemes_check {
      ltx_files
        .iter()
        .filter(|path| Self::is_ltx_scheme_path(path))
        .cloned()
        .collect()
    } else {
      Vec::new()
    };

    // In parallel, because this reads and parses every config in the tree to learn what it includes, and on an Anomaly
    // installation that is 3,129 files and the larger half of what opening a project costs. Each file is read on its
    // own and decides nothing about any other; the retained-document cache folds two threads missing one path into a
    // single load, so a config two others include is still read once.
    let (included, unreadable): (Vec<XrayLogicalPath>, Vec<XrayLogicalPath>) = ltx_files
      .par_iter()
      .map(|path| {
        let directory: PathBuf = path
          .parent()
          .map(|parent| PathBuf::from(parent.as_str()))
          .unwrap_or_default();

        match source.read_included(path.as_str()) {
          Ok(includes) => includes
            .iter()
            .map(|include| source.resolve(&directory, include))
            .collect::<XrfResult<Vec<Vec<PathBuf>>>>()
            .and_then(|resolved| {
              resolved
                .iter()
                .flatten()
                .map(|reached| Self::included_path(reached))
                .collect::<XrfResult<Vec<XrayLogicalPath>>>()
            })
            .map_err(|_| path),
          // Unreadable is not a failure of the walk: such a config becomes an entry point whatever else says, because
          // its own include list is unknown and the verifier has to reach it to report why.
          Err(_) => Err(path),
        }
      })
      .fold(
        || (Vec::new(), Vec::new()),
        |(mut included, mut unreadable), reached| {
          match reached {
            Ok(reached) => included.extend(reached),
            Err(path) => unreadable.push(path.clone()),
          }

          (included, unreadable)
        },
      )
      .reduce(
        || (Vec::new(), Vec::new()),
        |(mut included, mut unreadable), (reached, unread)| {
          included.extend(reached);
          unreadable.extend(unread);

          (included, unreadable)
        },
      );

    // Files that patch another config rather than standing alone. Under standard LTX there are none; under DLTX a
    // `mod_system_a.ltx` belongs to `system.ltx`, and verifying it on its own would report every override in it as
    // patching a section nothing declares.
    let attachments: Vec<String> = options.dialect.plan_attachments(
      &ltx_files
        .iter()
        .map(|path| String::from(path.as_str()))
        .collect::<Vec<String>>(),
      &source,
    )?;

    let mut ltx_file_entries: Vec<XrayLogicalPath> = Vec::new();
    let mut ltx_file_entries_failures: Vec<(XrayLogicalPath, XrayLogicalPath)> = Vec::new();

    // Filter our entries not included in other files and consider them entry-points.
    for ltx_file_path in ltx_files.iter() {
      // An unreadable config is an entry point whatever else says, because its own include list is unknown and the
      // verifier has to reach it to report why.
      if unreadable.contains(ltx_file_path) {
        ltx_file_entries.push(ltx_file_path.clone());

        continue;
      }

      if included.contains(ltx_file_path) || attachments.iter().any(|it| it == ltx_file_path.as_str()) {
        continue;
      }

      // To make checks more strict and consistent, verify typos with case-insensitive Windows OS.
      // Linux / sane logics fail when assuming that `ExAmPlE.TxT` is same as `example.txt`.
      // Part of strict checking because original gamedata has such failures.
      if options.is_strict_check
        && let Some(matching_path) = included
          .iter()
          .find(|it| it.as_str().eq_ignore_ascii_case(ltx_file_path.as_str()))
      {
        ltx_file_entries_failures.push((ltx_file_path.clone(), matching_path.clone()));
        continue;
      }

      ltx_file_entries.push(ltx_file_path.clone());
    }

    // Prepare big message with list of files referenced in case-insensitive check.
    if !ltx_file_entries_failures.is_empty() {
      return Err(XrfError::new_convert_error(format!(
        "Cannot read LTX project safely, detected case-insensitive #include statements:\n{}",
        ltx_file_entries_failures
          .iter()
          .map(|(first, second)| format!("  - {} incorrectly imported as {}", first, second))
          .collect::<Vec<_>>()
          .join("\n")
      )));
    }

    // Filter our entries not included in other files.
    let ltx_scheme_file_entries: Vec<XrayLogicalPath> = if options.is_with_schemes_check {
      ltx_scheme_files
        .iter()
        .filter_map(|it| if included.contains(it) { None } else { Some(it.clone()) })
        .collect()
    } else {
      Default::default()
    };

    let ltx_scheme_declarations: LtxSectionSchemes = if options.is_with_schemes_check {
      LtxSchemeParser::parse_from_vfs(&vfs, &scope, &ltx_scheme_file_entries)?
    } else {
      Default::default()
    };

    Ok(Self {
      counters,
      dialect: options.dialect.clone(),
      is_caching_resolutions: options.is_caching_resolutions,
      resolved: Mutex::default(),
      listings,
      ltx_attachments: attachments,
      ltx_file_entries,
      ltx_files,
      ltx_scheme_declarations,
      ltx_scheme_file_entries,
      ltx_scheme_files,
      root,
      scope,
      vfs,
    })
  }

  /// Returns LTX logical paths in scope, sorted for deterministic assembly.
  fn collect_logical_paths(vfs: &XrayVfs, scope: &XrayLookupScope) -> XrfResult<Vec<XrayLogicalPath>> {
    let mut paths: Vec<XrayLogicalPath> = Vec::new();

    for location in vfs.scoped(scope).list_entries() {
      // Already an engine identity, so nothing is re-validated here.
      if location.get_logical_path().has_extension(XrayExtension::Ltx) {
        paths.push(location.get_logical_path().clone());
      }
    }

    paths.sort();

    Ok(paths)
  }

  /// Converts a resolved include path into an engine identity.
  ///
  /// # Errors
  ///
  /// Returns an error for an invalid X-Ray logical path.
  fn included_path(path: &Path) -> XrfResult<XrayLogicalPath> {
    XrayLogicalPath::new(&path.to_string_lossy())
  }
}

impl LtxProject {
  /// Check if provided LTX file is scheme definition file.
  pub fn is_ltx_scheme_path(path: &XrayLogicalPath) -> bool {
    let name: &str = path.file_name();

    name == LTX_SCHEME_LTX_FILENAME || name.ends_with(LTX_SCHEME_EXTENSION)
  }

  /// Returns the VFS that resolves this project's files.
  pub fn vfs(&self) -> &XrayVfs {
    &self.vfs
  }

  pub fn scope(&self) -> &XrayLookupScope {
    &self.scope
  }

  /// Returns the filesystem path for a loose config, or the logical path for an archived or missing config.
  pub fn path_of(&self, logical_path: &XrayLogicalPath) -> PathBuf {
    self
      .physical_path_of(logical_path)
      .unwrap_or_else(|| PathBuf::from(logical_path.as_str()))
  }

  /// Returns a filesystem path when a loose config resolves.
  pub fn physical_path_of(&self, logical_path: &XrayLogicalPath) -> Option<PathBuf> {
    self
      .vfs
      .scoped(&self.scope)
      .find(logical_path.as_str())
      .ok()
      .flatten()
      .and_then(|location| location.to_physical_path())
  }

  /// Resolves one project config under this project's dialect, including includes and inheritance.
  ///
  /// # Errors
  ///
  /// Returns an error if the config is outside the scope or cannot be read or resolved.
  pub fn read_resolution(&self, logical_path: &XrayLogicalPath) -> XrfResult<Arc<LtxResolution>> {
    if !self.is_caching_resolutions {
      // Produced here rather than through the cell, so nothing is stored and nothing is kept.
      self.counters.record_resolution();

      return Ok(Arc::new(self.resolve(logical_path)?));
    }

    self.resolved_cell(logical_path).get_or_try_init(|| {
      let resolved: Arc<LtxResolution> = Arc::new(self.resolve(logical_path)?);

      // Produced once per root now that the cell admits one producer, so counting here counts what actually ran.
      self.counters.record_resolution();

      Ok(resolved)
    })
  }

  /// The cell holding one root's resolution, created on first ask.
  fn resolved_cell(&self, logical_path: &XrayLogicalPath) -> Arc<LtxResolvedRoot> {
    Arc::clone(
      self
        .resolved
        .lock()
        .expect("resolved config cache to not be poisoned")
        .entry(logical_path.clone())
        .or_default(),
    )
  }

  /// Reads one config as a parsed document, through whatever the mounted world retains.
  ///
  /// # Errors
  ///
  /// Returns an error when the config is not in scope, its bytes are not Windows-1251, or it will not parse.
  pub fn read_document(&self, logical_path: &XrayLogicalPath) -> XrfResult<Arc<LtxDocument>> {
    LtxVfsSource::new_counted(&self.vfs, &self.scope, &self.counters).read_document(logical_path.as_str())
  }

  /// Reads one config's bytes as authored, counted against this project.
  ///
  /// # Errors
  ///
  /// Returns an error when the config is not in scope.
  pub(crate) fn read_counted_bytes(&self, logical_path: &XrayLogicalPath) -> XrfResult<Vec<u8>> {
    let bytes: Vec<u8> = self.vfs.scoped(&self.scope).read_bytes(logical_path.as_str())?;

    self.counters.record_read(bytes.len() as u64);

    Ok(bytes)
  }

  /// Reads one root and applies this project's dialect to it, retaining nothing.
  fn resolve(&self, logical_path: &XrayLogicalPath) -> XrfResult<LtxResolution> {
    let source: LtxVfsSource = self.source();

    self
      .dialect
      .resolve(logical_path.as_str(), &source, LtxResolveRequest::plain())
  }

  /// Resolves one root and keeps everything the dialect can say about it, for a caller that has to explain a value.
  ///
  /// # Errors
  ///
  /// Returns an error if the config is outside the scope or cannot be read or resolved.
  pub fn resolve_explained(&self, logical_path: &XrayLogicalPath) -> XrfResult<LtxResolution> {
    let source: LtxVfsSource = self.source();

    self.counters.record_resolution();

    self
      .dialect
      .resolve(logical_path.as_str(), &source, LtxResolveRequest::with_provenance())
  }

  /// Drops one root's cached resolution, so the next read produces it again.
  pub fn forget_root(&self, logical_path: &XrayLogicalPath) -> bool {
    self
      .resolved
      .lock()
      .expect("resolved config cache to not be poisoned")
      .remove(logical_path)
      .is_some()
  }

  /// Which rules this project resolves its configs under.
  pub fn get_dialect(&self) -> &Arc<dyn LtxDialect> {
    &self.dialect
  }

  /// The port this project reads documents through, for a caller that has to read some itself.
  pub fn document_source(&self) -> impl LtxDocumentSource + '_ {
    self.source()
  }

  /// A counted source over this project's own scope, sharing its directory listings.
  fn source(&self) -> LtxVfsSource<'_> {
    LtxVfsSource::new_counted(&self.vfs, &self.scope, &self.counters).with_listings(&self.listings)
  }

  /// Resolves a config in the supplied scope using this project's dialect, without caching the result.
  ///
  /// # Errors
  ///
  /// Returns an error if the config cannot be read or resolved.
  pub fn read_full_in_scope(&self, scope: &XrayLookupScope, logical_path: &str) -> XrfResult<Ltx> {
    let source: LtxVfsSource = LtxVfsSource::new_counted(&self.vfs, scope, &self.counters);

    self.counters.record_resolution();

    Ok(
      self
        .dialect
        .resolve(logical_path, &source, LtxResolveRequest::plain())?
        .ltx,
    )
  }

  /// How much reading and parsing this project has done.
  pub fn get_read_counters(&self) -> LtxReadCountersSnapshot {
    self.counters.get_snapshot()
  }

  /// Prepends the project scope to a relative config path.
  ///
  /// # Errors
  ///
  /// Returns an error if the resulting logical path is invalid.
  pub fn config_path(&self, relative_path: &str) -> XrfResult<XrayLogicalPath> {
    match self.scope.get_prefix() {
      Some(prefix) => XrayLogicalPath::new(prefix)?.join(relative_path),
      None => XrayLogicalPath::new(relative_path),
    }
  }

  /// The engine identity of `system.ltx`, within this project's scope.
  ///
  /// # Errors
  ///
  /// Returns an error only if the resulting name stops being a valid logical path.
  pub fn system_ltx_path(&self) -> XrfResult<XrayLogicalPath> {
    self.config_path(SYSTEM_LTX_FILENAME)
  }

  /// Returns the display path for `system.ltx`.
  ///
  /// # Errors
  ///
  /// Returns an error if the scoped logical path is invalid.
  pub fn system_ltx_report_path(&self) -> XrfResult<PathBuf> {
    Ok(self.path_of(&self.system_ltx_path()?))
  }

  /// Reads `system.ltx` with its includes merged and inherited sections resolved.
  ///
  /// # Errors
  ///
  /// Returns an error when the config is not in scope or cannot be read or parsed.
  pub fn system_ltx(&self) -> XrfResult<Arc<LtxResolution>> {
    self.read_resolution(&self.system_ltx_path()?)
  }
}
