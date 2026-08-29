use std::borrow::Cow;
use std::fs;
use std::path::{Path, PathBuf};

use xrf_archive::ArchiveDescriptor;
use xrf_error::XrfResult;
use xrf_utils::format_path;

use crate::mount::xray_root::{find_implied_asset_root, implied_install_root};
use crate::path::{normalize, normalize_base};
use crate::{FsgameFile, XraySourceKind};

/// One source to mount before it is opened or indexed.
///
/// The plan stays plain data, so what to mount can be decided and inspected before anything is opened.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct XrayPlannedMount {
  /// Where the source lives.
  pub path: PathBuf,
  /// Logical base the source mounts at, empty for a root.
  pub base: String,
  /// Source implementation to construct for this path.
  pub kind: XraySourceKind,
  /// Caller-supplied diagnostic label, such as an `fsgame.ltx` alias.
  pub origin: String,
  /// Logical prefixes this source omits when indexed.
  ///
  /// Per source rather than per VFS, so an override tree can skip `textures\wip` while the installation beneath it keeps
  /// serving that prefix. Archives ignore it: nothing writes work-in-progress into a volume.
  pub ignored: Vec<String>,
}

/// An ordered list of sources to mount, highest priority first.
///
/// Constructors cover explicit or inferred installations, explicit or inferred asset roots, and partial subtrees.
///
/// Order is priority. [`Self::from_fsgame`] reverses declaration order, because the engine registers roots as declared and
/// later registrations overwrite earlier ones.
///
/// A plan is plain data, so it can be inspected or chained before anything is opened — which is how a mod tree layers
/// over an installation:
///
/// ```rust,no_run
/// use xrf_vfs::{XrayMountMode, XrayMountPlan, XrayVfs};
///
/// # fn main() -> xrf_error::XrfResult {
/// let plan: XrayMountPlan = XrayMountPlan::root("C:\\work\\my_mod")? // the mod wins every collision
///   .behind(XrayMountMode::Installation.plan("C:\\Games\\Anomaly")?) // the game answers the rest
///   .ignoring(&[String::from("textures\\wip")])?;
///
/// for planned in plan.get_mounts() {
///   println!("{} <- {} ({:?})", planned.base, format_path(planned.path), planned.kind);
/// }
///
/// let vfs: XrayVfs = XrayVfs::from_plan(&plan)?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct XrayMountPlan {
  mounts: Vec<XrayPlannedMount>,
}

impl XrayMountPlan {
  /// Creates an empty mount plan.
  pub fn new() -> Self {
    Self::default()
  }

  /// Plans one directory as a complete X-Ray root.
  pub fn root(path: impl AsRef<Path>) -> XrfResult<Self> {
    Self::new().with(path, "", "root")
  }

  /// Plans a partial directory at an explicit logical base.
  ///
  /// # Errors
  ///
  /// Returns an error when `base` is not a valid X-Ray logical path.
  pub fn subtree(path: impl AsRef<Path>, base: &str) -> XrfResult<Self> {
    Self::new().with(path, base, "subtree")
  }

  /// Plans the root implied by an asset path, or returns an empty plan when none is found.
  pub fn implied(asset: impl AsRef<Path>) -> XrfResult<Self> {
    match find_implied_asset_root(asset.as_ref()) {
      Some(root) => Self::new().with(root, "", "implied"),
      None => Ok(Self::new()),
    }
  }

  /// The X-Ray root a physical asset path sits under, if any — what [`Self::implied`] plans from.
  ///
  /// Walks upward from the asset and answers with the nearest ancestor holding both a `meshes` and a `textures`
  /// directory, so a gamedata tree nested inside another resolves against the one that contains the asset. Finding a
  /// root does not promise a reference resolves inside it; callers that need a resolvable root must fall through on a
  /// failed lookup rather than on a failed derivation.
  pub fn implied_root(asset: &Path) -> Option<PathBuf> {
    find_implied_asset_root(asset)
  }

  /// Plans the nearest installation containing an asset.
  ///
  /// Unlike [`Self::implied`], this uses an ancestor `fsgame.ltx`, so it also finds installations with an empty
  /// `gamedata/`. Returns an empty plan when no installation is found.
  ///
  /// # Errors
  ///
  /// Returns an error when an installation is found but its `fsgame.ltx` cannot be read, decoded, or parsed.
  pub fn implied_install(asset: impl AsRef<Path>) -> XrfResult<Self> {
    match implied_install_root(asset.as_ref()) {
      Some(install) => Self::from_fsgame(install),
      None => Ok(Self::new()),
    }
  }

  /// Plans the volume set a directory holds, as one archive source.
  ///
  /// The counterpart of [`Self::root`] for a directory of `.db` volumes rather than a loose tree: the same directory
  /// mounted loosely would answer for `textures.db0` as though it were an asset, and for none of the assets inside it.
  ///
  /// # Errors
  ///
  /// Returns an error when the mount cannot be planned at the root base.
  pub fn volumes(path: impl AsRef<Path>) -> XrfResult<Self> {
    Self::new().with_kind(path, "", "volumes", XraySourceKind::Archive)
  }

  /// Plans a whole installation from its `fsgame.ltx`.
  ///
  /// The plan includes the existing `$game_data$` directory and existing declared directories that directly contain a
  /// file whose extension starts with `db`. Other declared directories are omitted.
  ///
  /// # Errors
  ///
  /// Returns an error when `fsgame.ltx` cannot be read, decoded, or parsed.
  pub fn from_fsgame(install: impl AsRef<Path>) -> XrfResult<Self> {
    Self::from_fsgame_file(&FsgameFile::read(install)?)
  }

  /// Plans mounts from a parsed `fsgame.ltx` and the current filesystem contents.
  pub fn from_fsgame_file(fsgame: &FsgameFile) -> XrfResult<Self> {
    let mut plan: Self = Self::new();

    // Reversed: last declared wins, so it must be searched first.
    for declaration in fsgame.get_declarations().iter().rev() {
      let Some(path) = fsgame.resolve(&declaration.alias) else {
        continue;
      };

      if !path.is_dir() {
        log::info!(
          "Skipping fsgame alias {}: {} is not a directory",
          declaration.alias,
          format_path(&path)
        );

        continue;
      }

      // Always plan declared gamedata, even when empty: archive-only installations such as Anomaly still use it as the
      // writable override root.
      if Self::is_gamedata_root(fsgame, &path) {
        plan = plan.with_kind(path, "", &declaration.alias, XraySourceKind::Directory)?;

        continue;
      }

      if Self::holds_volumes(&path) {
        plan = plan.with_kind(path, "", &declaration.alias, XraySourceKind::Archive)?;
      }
    }

    Ok(plan)
  }

  /// Appends a directory mount.
  ///
  /// # Errors
  ///
  /// Returns an error when `base` is not a valid X-Ray logical path.
  pub fn with(self, path: impl AsRef<Path>, base: &str, origin: &str) -> XrfResult<Self> {
    self.with_kind(path, base, origin, XraySourceKind::Directory)
  }

  /// Appends a mount of the requested source kind.
  ///
  /// # Errors
  ///
  /// Returns an error when `base` is not a valid X-Ray logical path.
  pub fn with_kind(
    mut self,
    path: impl AsRef<Path>,
    base: &str,
    origin: &str,
    kind: XraySourceKind,
  ) -> XrfResult<Self> {
    self.mounts.push(XrayPlannedMount {
      base: normalize_base(base)?,
      ignored: Vec::new(),
      kind,
      origin: origin.to_string(),
      path: path.as_ref().to_path_buf(),
    });

    Ok(self)
  }

  /// Applies logical prefixes for every directory mount in this plan to omit.
  ///
  /// Set on the plan rather than passed to a mount call, so one `--ignore` reaches every source a mode planned without each
  /// caller threading it through. Archive mounts are unaffected.
  ///
  /// # Errors
  ///
  /// Returns an error when a prefix is not a valid X-Ray logical path.
  pub fn ignoring(mut self, ignored: &[String]) -> XrfResult<Self> {
    let ignored: Vec<String> = ignored
      .iter()
      .map(|prefix| normalize(prefix).map(Cow::into_owned))
      .collect::<XrfResult<_>>()?;

    for mount in &mut self.mounts {
      if mount.kind == XraySourceKind::Directory {
        mount.ignored = ignored.clone();
      }
    }

    Ok(self)
  }

  /// Appends another plan at lower priority.
  ///
  /// Duplicate paths are dropped, so chaining a fallback that happens to be the tree the asset already implied does not
  /// mount it twice.
  pub fn behind(mut self, other: Self) -> Self {
    for mount in other.mounts {
      if !self.mounts.iter().any(|existing| existing.path == mount.path) {
        self.mounts.push(mount);
      }
    }

    self
  }

  /// Returns mounts in priority order.
  pub fn get_mounts(&self) -> &[XrayPlannedMount] {
    &self.mounts
  }

  /// Returns whether the plan contains no mounts.
  pub fn is_empty(&self) -> bool {
    self.mounts.is_empty()
  }

  /// Returns the number of planned mounts.
  pub fn len(&self) -> usize {
    self.mounts.len()
  }

  /// Whether a directory holds archive volumes directly, which makes it an archive source.
  ///
  /// Only the immediate contents count, because fsgame declares each volume directory separately and the engine scans them
  /// non-recursively. Mounting such a directory as a loose source instead would register `textures.db0` as an addressable
  /// asset.
  pub fn holds_volumes(path: impl AsRef<Path>) -> bool {
    Self::holds_volumes_at(path.as_ref())
  }

  fn holds_volumes_at(path: &Path) -> bool {
    fs::read_dir(path).is_ok_and(|entries| entries.flatten().any(|entry| Self::is_volume(entry.path())))
  }

  /// Whether a file is an archive volume, asked of the crate that owns the format.
  pub fn is_volume(path: impl AsRef<Path>) -> bool {
    let path: &Path = path.as_ref();

    path.is_file() && ArchiveDescriptor::is_valid_db_path(path)
  }

  /// Whether a loose directory is the installation's gamedata root rather than one of its subdirectories.
  ///
  /// Subdirectory aliases such as `$game_meshes$` resolve inside gamedata, and mounting them alongside it would register
  /// every mesh twice - once as `meshes\x.ogf` and once as `x.ogf`.
  fn is_gamedata_root(fsgame: &FsgameFile, path: &Path) -> bool {
    fsgame.resolve("$game_data$").is_some_and(|gamedata| gamedata == path)
  }
}
