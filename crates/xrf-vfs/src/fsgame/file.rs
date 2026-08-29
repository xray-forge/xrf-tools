use std::fs;
use std::path::{Path, PathBuf};

use xrf_error::{XrfError, XrfResult};
use xrf_utils::{decode_bytes_to_string, format_path, new_windows1251_encoder};

use crate::fsgame::declaration::FsgameDeclaration;

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
  /// Standard filename for an installation's declared layout.
  pub const FILE_NAME: &'static str = "fsgame.ltx";

  /// Alias for the installation directory that anchors resolved paths.
  pub const ROOT_ALIAS: &'static str = "$fs_root$";

  /// Reads and parses the `fsgame.ltx` of an installation directory.
  ///
  /// The file is decoded as Windows-1251 so Cyrillic descriptions are preserved.
  ///
  /// # Errors
  ///
  /// Returns an error when the file cannot be read or decoded, or contains no valid declarations.
  pub fn read(root: impl AsRef<Path>) -> XrfResult<Self> {
    let root: &Path = root.as_ref();
    let path: PathBuf = root.join(Self::FILE_NAME);
    let bytes: Vec<u8> = fs::read(&path)
      .map_err(|error| XrfError::new_read_error(format!("failed to read {}: {error}", format_path(&path))))?;

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

  /// Resolves one alias, or returns `None` when its chain is undeclared, cyclic, or does not reach `$fs_root$`.
  pub fn resolve(&self, alias: &str) -> Option<PathBuf> {
    // Bounded by the declaration count, so a cycle terminates instead of recursing forever.
    let mut segments: Vec<&str> = Vec::new();
    let mut current: &str = alias;

    for _ in 0..=self.declarations.len() {
      if current == Self::ROOT_ALIAS {
        let mut path: PathBuf = self.root.clone();

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
}
