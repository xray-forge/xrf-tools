use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::path::Path;

use xrf_archive::{ArchiveFileDescriptor, ArchiveProject};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::path::{XrayLogicalPath, is_component_prefix, normalize_logical};
use crate::source::xray_asset_source::label_from_path;
use crate::{XrayAssetContainer, XrayAssetSource, XrayCollisionSite, XrayPathCollision, XraySourceKind};

/// Mounts an archive volume set as a read-only asset source.
///
/// Directory paths are scanned nonrecursively, matching `recurs = false` archive aliases and avoiding duplicate
/// subdirectory mounts.
///
/// [`ArchiveProject`] already merges a volume set into one name table with the later volume winning, which matches how the
/// engine registers them, so this adds only the logical-path keying a VFS lookup needs.
pub struct XrayArchiveSource {
  label: String,
  project: ArchiveProject,
  /// Normalized logical path to the key `project.files` stores.
  ///
  /// Archive headers keep names as authored, so the normalized form is derived once here rather than per lookup.
  entries: HashMap<String, String>,
  collisions: Vec<XrayPathCollision>,
}

impl XrayArchiveSource {
  /// Opens a volume set, or a single volume, at a path.
  pub fn read(path: impl AsRef<Path>) -> XrfResult<Self> {
    let path: &Path = path.as_ref();
    let source: Self = Self::from_project(ArchiveProject::new_shallow(path)?, label_from_path(path));

    log::info!(
      "Mounted {} archive entries from {}",
      source.entries.len(),
      format_path(path)
    );

    Ok(source)
  }

  /// Files a volume set's fold onto engine identities leaves unreachable, without mounting it.
  ///
  /// For a caller holding a volume set it has already read: `archive verify` reads every payload out of an
  /// [`ArchiveProject`] and then asks which of them no lookup can reach. Mounting the same path through [`Self::read`]
  /// would read every name table a second time, and discover volumes nonrecursively, so it would answer over a
  /// different volume set than the one just verified.
  ///
  /// The fold stays here rather than in `xrf-archive`, which cannot reach it: an archive keys entries by the name its
  /// header authored, and what those names fold to is the engine identity this crate's `path` module is the sole owner
  /// of.
  pub fn list_collisions_of(project: &ArchiveProject) -> Vec<XrayPathCollision> {
    Self::index(project).1
  }

  /// Keys an already-read volume set by engine identity.
  ///
  /// Separate from [`Self::read`] because a case-only collision inside one volume cannot exist on a case-insensitive
  /// filesystem, so the ordering rule below is only reachable from a name table built in a test.
  fn from_project(project: ArchiveProject, label: String) -> Self {
    let (entries, collisions) = Self::index(&project);

    Self {
      collisions,
      entries,
      label,
      project,
    }
  }

  /// Folds authored names to engine identities, recording the entries the fold leaves unreachable.
  ///
  /// **Last wins**, as `CLocatorAPI::Register` resolves it: it lower-cases a name before its name-table lookup and
  /// overwrites on a hit, so the later registration answers (`xray-16/src/xrCore/LocatorAPI.cpp`). Later here means the
  /// later volume in [`ArchiveProject`] order, which is the precedence it already applies to exact duplicates, so a patch
  /// volume overrides `Textures\A.DDS` exactly as it overrides `textures\a.dds`.
  ///
  /// Within one volume the engine order is the header chunk order, and the authored name stands in for it. That is the
  /// same order for anything `xrCompress` or `ArchivePacker` wrote, since both emit entries sorted by name.
  ///
  /// A collision is recorded rather than refused: a person has to be able to open a volume set to learn what is wrong
  /// with it, and the engine does not refuse it either.
  // todo: Header order is dropped by the reader, so the within-volume rule is an approximation.
  fn index(project: &ArchiveProject) -> (HashMap<String, String>, Vec<XrayPathCollision>) {
    let ranks: HashMap<&Path, usize> = project
      .archives
      .iter()
      .enumerate()
      .map(|(rank, archive)| (archive.path.as_path(), rank))
      .collect();

    let mut entries: HashMap<String, String> = HashMap::with_capacity(project.files.len());
    let mut collisions: Vec<XrayPathCollision> = Vec::new();

    for (name, descriptor) in &project.files {
      if descriptor.is_directory {
        continue;
      }

      let Ok(normalized) = normalize_logical(name).inspect_err(|error| {
        log::warn!("Skipping archive entry '{name}': {error}");
      }) else {
        continue;
      };

      match entries.entry(normalized) {
        Entry::Vacant(slot) => {
          slot.insert(name.clone());
        }
        Entry::Occupied(mut slot) => {
          let incumbent: String = slot.get().clone();
          let logical_path: XrayLogicalPath = XrayLogicalPath::from_normalized(slot.key().clone());
          let is_replacing: bool =
            (Self::get_rank_of(&ranks, descriptor), name.as_str()) > Self::get_precedence(project, &ranks, &incumbent);

          let (kept, unreachable) = if is_replacing {
            (name.as_str(), incumbent.as_str())
          } else {
            (incumbent.as_str(), name.as_str())
          };

          collisions.push(XrayPathCollision {
            kept: Self::get_new_site(project, kept),
            logical_path,
            unreachable: Self::get_new_site(project, unreachable),
          });

          if is_replacing {
            slot.insert(name.clone());
          }
        }
      }
    }

    (entries, collisions)
  }

  /// Which volume of the set holds an entry, as a position in merge order.
  fn get_rank_of(ranks: &HashMap<&Path, usize>, descriptor: &ArchiveFileDescriptor) -> usize {
    ranks.get(descriptor.source.as_path()).copied().unwrap_or_default()
  }

  /// What decides between two entries folding to one identity: volume order first, then the authored name.
  fn get_precedence<'a>(project: &ArchiveProject, ranks: &HashMap<&Path, usize>, name: &'a str) -> (usize, &'a str) {
    let rank: usize = project
      .files
      .get(name)
      .map_or_else(usize::default, |descriptor| Self::get_rank_of(ranks, descriptor));

    (rank, name)
  }

  /// Where an authored entry sits, for a collision diagnostic.
  fn get_new_site(project: &ArchiveProject, name: &str) -> XrayCollisionSite {
    XrayCollisionSite::Archived {
      name: name.to_owned(),
      volume: project
        .files
        .get(name)
        .map(|descriptor| descriptor.source.clone())
        .unwrap_or_default(),
    }
  }

  /// The merged volume set behind this source, for consumers that need descriptor-level detail.
  pub fn get_project(&self) -> &ArchiveProject {
    &self.project
  }
}

/// Written by hand rather than derived, because a derived one would print the whole name table - 17,188 assets for
/// Anomaly's texture volumes alone. What identifies a mount is which volume set it is and how much it holds.
impl Debug for XrayArchiveSource {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
    formatter
      .debug_struct("XrayArchiveSource")
      .field("label", &self.label)
      .field("root", &self.project.root)
      .field("entries", &self.entries.len())
      .finish()
  }
}

impl XrayAssetSource for XrayArchiveSource {
  fn get_label(&self) -> &str {
    &self.label
  }

  fn get_kind(&self) -> XraySourceKind {
    XraySourceKind::Archive
  }

  /// Always false. Writing into a volume is out of scope; a caller wanting to change an archived asset writes a loose
  /// override instead.
  fn is_writable(&self) -> bool {
    false
  }

  fn get_root_path(&self) -> &Path {
    &self.project.root
  }

  fn contains(&self, path: &str) -> bool {
    self.entries.contains_key(path)
  }

  fn locate(&self, path: &str) -> Option<XrayAssetContainer> {
    self.entries.contains_key(path).then(|| XrayAssetContainer::Archive {
      path: self.project.root.clone(),
    })
  }

  fn read(&self, path: &str) -> XrfResult<Vec<u8>> {
    let Some(name) = self.entries.get(path) else {
      // Absent, not unreadable: the distinction lets a caller fall back rather than fail.
      return Err(XrfError::new_not_found_error(format!(
        "no archive entry '{path}' in {}",
        self.label
      )));
    };

    self.project.read_file_bytes(name)
  }

  /// Answers from the volume's name table, so no entry is decompressed to learn its size.
  fn get_size(&self, path: &str) -> Option<u64> {
    self
      .entries
      .get(path)
      .and_then(|name| self.project.files.get(name))
      .map(|descriptor| u64::from(descriptor.size_real))
  }

  fn write(&self, path: &str, _bytes: &[u8]) -> XrfResult<()> {
    Err(XrfError::new_read_error(format!(
      "cannot write '{path}': archive '{}' is read only",
      self.label
    )))
  }

  /// Always fails. A volume cannot gain an entry; an override belongs in a loose mount in front of it.
  fn create(&self, path: &str, _bytes: &[u8]) -> XrfResult<()> {
    Err(XrfError::new_read_error(format!(
      "cannot create '{path}': archive '{}' is read only",
      self.label
    )))
  }

  fn list_entries<'a>(&'a self, prefix: Option<&'a str>) -> Box<dyn Iterator<Item = String> + 'a> {
    Box::new(
      self
        .entries
        .keys()
        .filter(move |path| prefix.is_none_or(|prefix| is_component_prefix(path, prefix)))
        .cloned(),
    )
  }

  fn get_collisions(&self) -> &[XrayPathCollision] {
    &self.collisions
  }
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;
  use std::path::{Path, PathBuf};

  use xrf_archive::{ArchiveDescriptor, ArchiveFileDescriptor, ArchiveProject, ArchiveProjectReadPolicy};

  use crate::{XrayAssetSource, XrayCollisionSite, XrayPathCollision};

  use super::XrayArchiveSource;

  const BASE: &str = "C:\\game\\db\\base.db0";
  const PATCH: &str = "C:\\game\\db\\patch.db1";

  /// A merged name table, as [`ArchiveProject`] hands one over: volumes in merge order, entries keyed as authored.
  ///
  /// Built rather than packed because a case-only pair cannot exist in one directory on a case-insensitive filesystem,
  /// which is the only way to reach the within-volume half of the rule. The packed round trip is covered by
  /// `xrf-pack`'s `asset_source_tests`.
  fn project(volumes: &[&str], files: &[(&str, &str, u32)]) -> ArchiveProject {
    ArchiveProject {
      archives: volumes
        .iter()
        .map(|volume| ArchiveDescriptor {
          created_at: None,
          files: HashMap::new(),
          modified_at: None,
          output_root_path: PathBuf::from("gamedata"),
          path: PathBuf::from(volume),
        })
        .collect(),
      files: files
        .iter()
        .map(|(volume, name, size)| {
          (
            (*name).to_owned(),
            ArchiveFileDescriptor::new(0, (*name).to_owned(), 0, *size, *size)
              .with_archive_paths(Path::new(volume), Path::new("gamedata")),
          )
        })
        .collect(),
      read_policy: ArchiveProjectReadPolicy::default(),
      root: PathBuf::from("C:\\game\\db"),
      size_real: 0,
    }
  }

  fn source(volumes: &[&str], files: &[(&str, &str, u32)]) -> XrayArchiveSource {
    XrayArchiveSource::from_project(project(volumes, files), String::from("db"))
  }

  /// Asserts a site names one authored entry of one volume.
  fn assert_site(site: &XrayCollisionSite, expected_volume: &str, expected_name: &str) {
    match site {
      XrayCollisionSite::Archived { volume, name } => {
        assert_eq!(volume, Path::new(expected_volume));
        assert_eq!(name, expected_name);
      }
      XrayCollisionSite::Loose(path) => panic!("archived entry expected, got loose {}", path.display()),
    }
  }

  #[test]
  fn a_later_volume_wins_a_case_folded_collision() {
    // Volume order beats the name tiebreak, so a patch overrides `Textures\A.DDS` exactly as it overrides
    // `textures\a.dds` - otherwise a patch would win or lose by how the two happened to be spelled.
    let source: XrayArchiveSource = source(
      &[BASE, PATCH],
      &[(BASE, "textures\\a.dds", 10), (PATCH, "Textures\\A.DDS", 20)],
    );

    assert_eq!(
      source.get_size("textures\\a.dds"),
      Some(20),
      "the patch volume answers, though its name sorts first"
    );

    let collisions: &[XrayPathCollision] = source.get_collisions();

    assert_eq!(collisions.len(), 1);
    assert_eq!(collisions[0].logical_path.as_str(), "textures\\a.dds");
    assert_site(&collisions[0].kept, PATCH, "Textures\\A.DDS");
    assert_site(&collisions[0].unreachable, BASE, "textures\\a.dds");
  }

  #[test]
  fn the_last_authored_name_wins_inside_one_volume() {
    // One volume has no precedence to appeal to, so the authored name orders it - standing in for the header order
    // the engine registers in, which is the same order for anything xrCompress or ArchivePacker wrote.
    let source: XrayArchiveSource = source(&[BASE], &[(BASE, "Textures\\A.DDS", 10), (BASE, "textures\\a.dds", 20)]);

    assert_eq!(source.get_size("textures\\a.dds"), Some(20));

    let collisions: &[XrayPathCollision] = source.get_collisions();

    assert_eq!(collisions.len(), 1);
    assert_site(&collisions[0].kept, BASE, "textures\\a.dds");
    assert_site(&collisions[0].unreachable, BASE, "Textures\\A.DDS");
  }

  #[test]
  fn the_winner_does_not_depend_on_name_table_iteration_order() {
    // The defect this pins: two `HashMap`s over the same keys iterate differently, so a fold that let insertion order
    // decide answered from a different volume run to run.
    for _ in 0..16 {
      let source: XrayArchiveSource = source(
        &[BASE, PATCH],
        &[
          (BASE, "textures\\a.dds", 10),
          (PATCH, "Textures\\A.DDS", 20),
          (BASE, "textures\\b.dds", 30),
          (BASE, "configs\\system.ltx", 40),
        ],
      );

      assert_eq!(source.get_size("textures\\a.dds"), Some(20));
      assert_eq!(source.get_collisions().len(), 1);
    }
  }

  #[test]
  fn a_name_table_that_does_not_fold_reports_nothing() {
    let source: XrayArchiveSource = source(
      &[BASE, PATCH],
      &[
        (BASE, "textures\\a.dds", 10),
        (PATCH, "textures\\b.dds", 20),
        (BASE, "configs\\system.ltx", 30),
      ],
    );

    assert!(source.get_collisions().is_empty());
    assert_eq!(source.list_entries(None).count(), 3);
  }

  #[test]
  fn a_directory_row_never_collides_with_the_directory_it_names() {
    // Directory rows are not entries. Counting one would both inflate the listing and report a collision against the
    // trailing-separator form of a name that is already indexed.
    let source: XrayArchiveSource = source(
      &[BASE],
      &[
        (BASE, "Textures\\", 0),
        (BASE, "textures", 10),
        (BASE, "textures\\a.dds", 20),
      ],
    );

    assert!(source.get_collisions().is_empty());
    assert_eq!(source.get_size("textures"), Some(10));
  }
}
