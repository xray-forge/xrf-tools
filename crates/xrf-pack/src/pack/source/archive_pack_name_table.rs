//! The engine-name table one packing run registers: what makes each collected name a row the engine can read.

use std::fs;
use std::path::Path;

use xrf_error::{XrfError, XrfResult};
use xrf_vfs::XrayLogicalPath;

use crate::pack::config::ArchivePackConfig;
use crate::pack::source::ArchivePackEntry;
use crate::pack::source::ArchivePackNameCollision;

/// Every file and directory one packing run writes, under the name `CLocatorAPI::Register` folds it to, once each, in
/// the order the engine's own table iterates.
///
/// `Register` lower-cases a name before its lookup in `m_files` and overwrites on a hit, so two authored names with one
/// folded form are one file to the engine and only the payload registered last is ever readable. xrCompress never
/// wrote such a pair, because its file list came out of that same folded table; the packing walk reads the host tree
/// directly, which on a case-sensitive filesystem can hold both. Registering here, before any volume exists, is where
/// a person can still rename one.
///
/// Rows keep their authored spelling. The folded names live only as long as registration, so what the write phase
/// holds costs what the walk found.
#[derive(Debug)]
pub(crate) struct ArchivePackNameTable {
  files: Vec<ArchivePackEntry>,
  /// Written as zero-payload rows with a trailing separator, so any single volume lists the whole tree.
  directories: Vec<String>,
}

impl ArchivePackNameTable {
  /// Register what the walk found, refusing a set the engine would fold into fewer files.
  ///
  /// One file reached under two spellings — `[include_files]` naming what the walk also found, or a case-insensitive
  /// host — is one payload and registers once, under the spelling that sorts first. Two different files under one
  /// engine name are refused, every such name reported at once so a person fixes them in one pass. A directory row
  /// has no payload to lose, so a folded pair of them simply registers once.
  ///
  /// # Errors
  ///
  /// Returns an invalid error for a name the engine cannot address, and for any engine name more than one file claims.
  pub(crate) fn register(
    config: &ArchivePackConfig,
    files: Vec<ArchivePackEntry>,
    directories: Vec<String>,
  ) -> XrfResult<Self> {
    let (files, collisions) = Self::register_files(files)?;

    if !collisions.is_empty() {
      return Err(ArchivePackNameCollision::describe_refusal(config, &collisions));
    }

    Ok(Self {
      files,
      directories: Self::register_directories(directories)?,
    })
  }

  pub(crate) fn get_files(&self) -> &[ArchivePackEntry] {
    &self.files
  }

  pub(crate) fn get_directories(&self) -> &[String] {
    &self.directories
  }

  /// Register the files by engine name: one row per name, and every name more than one file claims.
  ///
  /// Sorted by engine name so each name arrives as one run, and unstably because two rows equal in both keys are one
  /// name twice, of which the run keeps whichever came first; a stable sort would buy nothing but the buffer it
  /// allocates.
  fn register_files(files: Vec<ArchivePackEntry>) -> XrfResult<(Vec<ArchivePackEntry>, Vec<ArchivePackNameCollision>)> {
    let mut keyed: Vec<(String, ArchivePackEntry)> = files
      .into_iter()
      .map(|file| Ok((to_engine_name(&file.name)?, file)))
      .collect::<XrfResult<_>>()?;

    keyed.sort_unstable_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.name.cmp(&right.1.name)));

    let mut registration: ArchivePackNameRegistration = ArchivePackNameRegistration::with_capacity(keyed.len());

    for (engine_name, file) in keyed {
      registration.claim(engine_name, file);
    }

    Ok(registration.finish())
  }

  /// Register the directories by engine name, keeping the first spelling of each.
  fn register_directories(directories: Vec<String>) -> XrfResult<Vec<String>> {
    let mut keyed: Vec<(String, String)> = directories
      .into_iter()
      .map(|directory| Ok((to_engine_name(&directory)?, directory)))
      .collect::<XrfResult<_>>()?;

    keyed.sort_unstable_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
    keyed.dedup_by(|later, earlier| later.0 == earlier.0);

    Ok(keyed.into_iter().map(|(_, directory)| directory).collect())
  }
}

/// Registration in progress over sorted rows.
///
/// Private to the table it builds: a half-registered set has no meaning of its own, and the run-detection it performs
/// is only correct against input the table has already sorted.
struct ArchivePackNameRegistration {
  rows: Vec<ArchivePackEntry>,
  collisions: Vec<ArchivePackNameCollision>,
  /// The distinct files claiming the engine name under registration, which sorted input hands over as one run.
  claimants: Vec<(String, ArchivePackEntry)>,
}

impl ArchivePackNameRegistration {
  /// Room for every row up front, so registering never reallocates while the keyed rows are still held beside it.
  fn with_capacity(rows: usize) -> Self {
    Self {
      rows: Vec::with_capacity(rows),
      collisions: Vec::new(),
      claimants: Vec::new(),
    }
  }

  /// Take the next row: the same name again, or one file more claiming the name under registration, or a new name.
  fn claim(&mut self, engine_name: String, file: ArchivePackEntry) {
    if self
      .claimants
      .first()
      .is_some_and(|(claimed, _)| *claimed != engine_name)
    {
      self.settle();
    }

    let is_registered: bool = self
      .claimants
      .iter()
      .any(|(_, claimant)| claimant.name == file.name || is_same_file(&claimant.path, &file.path));

    if !is_registered {
      self.claimants.push((engine_name, file));
    }
  }

  /// Close the name under registration: one claimant is its row, several are a collision.
  fn settle(&mut self) {
    if self.claimants.len() > 1 {
      self.collisions.push(ArchivePackNameCollision {
        engine_name: self.claimants[0].0.clone(),
        spellings: self.claimants.drain(..).map(|(_, file)| file.name).collect(),
      });
    } else if let Some((_, file)) = self.claimants.pop() {
      self.rows.push(file);
    }
  }

  fn finish(mut self) -> (Vec<ArchivePackEntry>, Vec<ArchivePackNameCollision>) {
    self.settle();

    (self.rows, self.collisions)
  }
}

/// The name `Register` folds an authored one to.
fn to_engine_name(name: &str) -> XrfResult<String> {
  XrayLogicalPath::normalize(name).map_err(|error| {
    XrfError::new_invalid_error(format!(
      "Packing source entry '{name}' is not a name the engine can address: {error}"
    ))
  })
}

/// Whether two host paths reach one file.
///
/// Asked only of two files claiming one engine name, so its cost is paid per collision rather than per row. A path
/// that cannot be resolved is not proven to be the same file, and the claim stands; the write phase would have failed
/// on that path anyway.
fn is_same_file(left: &Path, right: &Path) -> bool {
  match (fs::canonicalize(left), fs::canonicalize(right)) {
    (Ok(left), Ok(right)) => left == right,
    _ => false,
  }
}

#[cfg(test)]
mod tests {
  use std::fs;
  use std::path::{Path, PathBuf};

  use xrf_error::XrfResult;
  use xrf_test_utils::utils::build_absolute_generated_test_resource_path;

  use super::ArchivePackNameTable;
  use crate::pack::config::ArchivePackConfig;
  use crate::pack::source::ArchivePackEntry;

  /// Rows the way the walk would hand them over, against host paths that need not exist.
  ///
  /// No case-insensitive filesystem can hold a case-only pair in one directory, so the pair is authored here rather
  /// than on disk. Registration is a function over names and paths, which is what makes that possible.
  fn entries(names: &[&str]) -> Vec<ArchivePackEntry> {
    names
      .iter()
      .map(|name| ArchivePackEntry {
        name: String::from(*name),
        path: Path::new("gamedata").join(name.replace('\\', "/")),
      })
      .collect()
  }

  fn register(files: &[&str], directories: &[&str]) -> XrfResult<ArchivePackNameTable> {
    ArchivePackNameTable::register(
      &ArchivePackConfig::new("gamedata", "db", "packed"),
      entries(files),
      directories.iter().map(|name| String::from(*name)).collect(),
    )
  }

  fn file_names(table: &ArchivePackNameTable) -> Vec<&str> {
    table.get_files().iter().map(|file| file.name.as_str()).collect()
  }

  #[test]
  fn a_case_only_pair_of_distinct_files_is_refused_naming_both() {
    let message: String = register(&["textures\\wall.dds", "textures\\A.dds", "textures\\a.dds"], &[])
      .expect_err("a pair the engine folds is refused")
      .to_string();

    assert!(
      message.contains("'textures\\A.dds' and 'textures\\a.dds' both register as 'textures\\a.dds'"),
      "'{message}' names both spellings and the engine name"
    );
    assert!(message.contains("1 engine name(s)"), "'{message}' counts the names");
  }

  #[test]
  fn three_spellings_of_one_engine_name_are_one_refusal() {
    let message: String = register(&["textures\\a.dds", "textures\\A.DDS", "textures\\A.dds"], &[])
      .expect_err("three files under one name are refused")
      .to_string();

    assert!(
      message.contains("'textures\\A.DDS', 'textures\\A.dds' and 'textures\\a.dds' all register as 'textures\\a.dds'"),
      "'{message}' reports one name claimed three times, not two pairs"
    );
    assert!(message.contains("1 engine name(s)"), "'{message}' counts the names");
  }

  #[test]
  fn every_claimed_name_is_reported_so_one_run_shows_them_all() {
    let message: String = register(
      &[
        "configs\\System.ltx",
        "configs\\system.ltx",
        "Textures\\a.dds",
        "textures\\a.dds",
      ],
      &[],
    )
    .expect_err("both names are refused")
    .to_string();

    for expected in ["'configs\\System.ltx'", "'Textures\\a.dds'", "2 engine name(s)"] {
      assert!(message.contains(expected), "'{message}' names {expected}");
    }
  }

  #[test]
  fn one_file_named_twice_registers_once() {
    // `[include_files]` naming a file the directory walk also reached: one payload, one row.
    let table: ArchivePackNameTable = register(&["configs\\system.ltx", "configs\\system.ltx", "configs\\a.ltx"], &[])
      .expect("a repeated name is not a collision");

    assert_eq!(file_names(&table), ["configs\\a.ltx", "configs\\system.ltx"]);
  }

  #[test]
  fn two_spellings_of_one_host_file_register_once_under_the_first() {
    // What a case-insensitive host produces when `[include_files]` spells a walked file differently: both names reach
    // one file, so the engine would get the same bytes either way and there is nothing to refuse.
    let root: PathBuf =
      build_absolute_generated_test_resource_path("archive_pack_name_table/two_spellings_of_one_host_file");
    let path: PathBuf = root.join("system.ltx");

    fs::create_dir_all(&root).expect("scratch directory");
    fs::write(&path, b"[section]").expect("scratch file");

    let files: Vec<ArchivePackEntry> = ["configs\\system.ltx", "Configs\\System.ltx"]
      .into_iter()
      .map(|name| ArchivePackEntry {
        name: String::from(name),
        path: path.clone(),
      })
      .collect();

    let table: ArchivePackNameTable =
      ArchivePackNameTable::register(&ArchivePackConfig::new(&root, "db", "packed"), files, Vec::new())
        .expect("one file under two spellings is one payload");

    assert_eq!(file_names(&table), ["Configs\\System.ltx"]);
  }

  #[test]
  fn files_come_out_in_the_order_the_engine_table_iterates() {
    // Byte order would put `Zeta` before `alpha`; the engine's table is keyed by the folded name.
    let table: ArchivePackNameTable =
      register(&["configs\\Zeta.ltx", "configs\\beta.ltx", "configs\\alpha.ltx"], &[]).expect("distinct names");

    assert_eq!(
      file_names(&table),
      ["configs\\alpha.ltx", "configs\\beta.ltx", "configs\\Zeta.ltx"]
    );
  }

  #[test]
  fn a_folded_pair_of_directories_registers_once() {
    // Nothing is lost when two directory rows fold together, so the pair is not a collision; one row lists the folder.
    let table: ArchivePackNameTable =
      register(&[], &["textures", "Textures", "configs", "configs"]).expect("directory rows never collide");

    assert_eq!(table.get_directories(), ["configs", "Textures"]);
  }

  #[test]
  fn a_name_the_engine_cannot_address_is_refused() {
    let message: String = register(&["configs\\..\\system.ltx"], &[])
      .expect_err("a traversal is not an engine name")
      .to_string();

    assert!(
      message.contains("'configs\\..\\system.ltx'"),
      "'{message}' names the entry"
    );
  }
}
