use std::fs;
use std::path::{Path, PathBuf};

use xrf_error::{XrfError, XrfResult};
use xrf_utils::{decode_bytes_to_string, format_path, new_windows1251_encoder};

use crate::mount::fsgame::FsgameDeclaration;

/// One installation's declared directory layout, in declaration order.
///
/// Declarations remain in file order because the engine registers them in sequence and later registrations override
/// earlier ones.
#[derive(Clone, Debug)]
pub struct FsgameFile {
  root: PathBuf,
  declarations: Vec<FsgameDeclaration>,
}

impl FsgameFile {
  /// Names an installation declares its layout under, in the order a directory is searched for one.
  ///
  /// `fsgame.ltx` first, which every engine xrf targets reads; then each Enhanced Edition's own name.
  pub const FILE_NAMES: [&'static str; 4] = ["fsgame.ltx", "fsgame_cop.ltx", "fsgame_cs.ltx", "fsgame_soc.ltx"];

  /// Alias for the installation directory that anchors resolved paths.
  pub const ROOT_ALIAS: &'static str = "$fs_root$";

  /// The fsgame file a directory declares an installation with, if it holds one.
  pub fn find(directory: impl AsRef<Path>) -> Option<PathBuf> {
    let directory: &Path = directory.as_ref();

    Self::FILE_NAMES
      .iter()
      .map(|name| directory.join(name))
      .find(|path| path.is_file())
  }

  /// Reads and parses the fsgame file of an installation directory.
  ///
  /// # Errors
  ///
  /// Returns an error when the directory holds no fsgame file, or it cannot be read or parsed.
  pub fn read(root: impl AsRef<Path>) -> XrfResult<Self> {
    let root: &Path = root.as_ref();
    let path: PathBuf = Self::find(root).ok_or_else(|| {
      XrfError::new_read_error(format!(
        "{} holds none of {}",
        format_path(root),
        Self::FILE_NAMES.join(", ")
      ))
    })?;

    Self::read_file(path)
  }

  /// Reads and parses one fsgame file, anchoring `$fs_root$` at the directory holding it.
  ///
  /// The file is decoded as Windows-1251 so Cyrillic descriptions are preserved.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be read or decoded, or contains no valid declarations.
  pub fn read_file(path: impl AsRef<Path>) -> XrfResult<Self> {
    let path: &Path = path.as_ref();
    let root: &Path = path
      .parent()
      .ok_or_else(|| XrfError::new_read_error(format!("{} has no parent directory", format_path(path))))?;
    let bytes: Vec<u8> = fs::read(path)
      .map_err(|error| XrfError::new_read_error(format!("failed to read {}: {error}", format_path(path))))?;

    Self::parse(root, &decode_bytes_to_string(&bytes, new_windows1251_encoder())?)
  }

  /// Parses declarations out of file contents.
  ///
  /// Declaration candidates that fail to parse are logged and skipped.
  ///
  /// # Errors
  ///
  /// Returns an error when no valid declarations remain.
  pub fn parse(root: impl AsRef<Path>, contents: &str) -> XrfResult<Self> {
    let declarations: Vec<FsgameDeclaration> = contents
      .lines()
      .filter(|line| FsgameDeclaration::is_declaration(line))
      .filter_map(|line| {
        FsgameDeclaration::parse(line)
          .inspect_err(|error| log::warn!("Skipping fsgame line: {error}"))
          .ok()
      })
      .collect();

    if declarations.is_empty() {
      return Err(XrfError::new_invalid_error("fsgame declares no aliases"));
    }

    Ok(Self {
      declarations,
      root: root.as_ref().to_path_buf(),
    })
  }

  /// Returns the installation directory represented by `$fs_root$`.
  pub fn get_root(&self) -> &Path {
    &self.root
  }

  /// Returns declarations in file order.
  pub fn get_declarations(&self) -> &[FsgameDeclaration] {
    &self.declarations
  }

  /// Finds an alias by its exact spelling.
  pub fn find_declaration(&self, alias: &str) -> Option<&FsgameDeclaration> {
    self.declarations.iter().find(|it| it.alias == alias)
  }

  /// Resolves one alias, or returns `None` when its chain is undeclared or cyclic.
  ///
  /// A chain ends at `$fs_root$`, the installation directory, or at a root column naming an absolute directory instead
  /// of an alias, which the engine's locator takes as that directory. An alias it never reaches is unresolved, as
  /// `$app_data_root$` is in an Enhanced Edition file whose engine supplies it.
  pub fn resolve(&self, alias: &str) -> Option<PathBuf> {
    // Bounded by the declaration count, so a cycle terminates instead of recursing forever.
    let mut segments: Vec<&str> = Vec::new();
    let mut current: &str = alias;

    for _ in 0..=self.declarations.len() {
      let base: Option<PathBuf> = if current == Self::ROOT_ALIAS {
        Some(self.root.clone())
      } else {
        Self::to_absolute_root(current)
      };

      if let Some(mut path) = base {
        for segment in segments.iter().rev() {
          path.push(segment);
        }

        return Some(path);
      }

      let declaration: &FsgameDeclaration = self.find_declaration(current)?;

      if let Some(addition) = declaration.get_addition_segment() {
        segments.push(addition);
      }

      current = &declaration.root_alias;
    }

    None
  }

  /// The directory a root column names when it spells an absolute path rather than an alias.
  fn to_absolute_root(root: &str) -> Option<PathBuf> {
    let path: &Path = Path::new(root);

    (!root.starts_with('$') && path.is_absolute()).then(|| path.to_path_buf())
  }
}
