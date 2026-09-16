use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::path::Path;
use std::sync::Arc;

use xrf_archive::{ArchiveDescriptor, ArchiveFileDescriptor, ArchiveProject};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::format_path;

use crate::path::{XrayLogicalPath, is_component_prefix, normalize_logical};
use crate::source::xray_asset_source::label_from_path;
use crate::{
  XrayAssetContainer, XrayAssetSource, XrayCollisionSite, XrayDeclaredRoot, XrayPathCollision, XraySourceKind,
  XraySourceOverride, XraySourceShadowedCopy,
};

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
  /// Engine identity to the authored name it folds from.
  ///
  /// The key is this source's own value: a normalized logical path exists nowhere else. The value is the archive's
  /// name for the same entry, shared with the descriptor that owns it rather than cloned, since it is only ever used
  /// to address that descriptor again.
  entries: HashMap<String, IndexedEntry>,
  collisions: Vec<XrayPathCollision>,
  /// Copies a later volume overrode, in engine-path order.
  shadowed: Vec<XraySourceShadowedCopy>,
}

/// What one engine identity resolves to inside a volume set.
struct IndexedEntry {
  /// The archive's own name for the entry, shared with the descriptor that owns it rather than cloned.
  name: Arc<str>,
  /// Volume holding it, as a position in [`ArchiveProject::archives`].
  volume: u32,
}

impl IndexedEntry {
  fn of(descriptor: &ArchiveFileDescriptor, name: &Arc<str>) -> Self {
    Self {
      name: Arc::clone(name),
      volume: descriptor.volume,
    }
  }
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

  /// Every engine path a volume set answers with more than one copy, without mounting it.
  pub fn list_overrides_of(project: &ArchiveProject) -> Vec<XraySourceOverride> {
    let (entries, _, shadowed) = Self::index(project);
    let mut overrides: Vec<XraySourceOverride> = Vec::new();

    // `shadowed` is already ordered by engine path, so the copies of one path arrive together and in precedence order.
    for copy in shadowed {
      match overrides.last_mut() {
        Some(previous) if previous.logical_path == copy.logical_path => previous.shadowed.push(copy),
        _ => {
          let Some(winner) = entries
            .get(&copy.logical_path)
            .and_then(|entry| project.files.get(&entry.name))
          else {
            continue;
          };

          overrides.push(XraySourceOverride {
            container: Self::to_volume_container(project, winner.volume),
            logical_path: copy.logical_path.clone(),
            shadowed: vec![copy],
            size: u64::from(winner.size_real),
          });
        }
      }
    }

    overrides
  }

  /// Keys an already-read volume set by engine identity.
  ///
  /// Separate from [`Self::read`] because a case-only collision inside one volume cannot exist on a case-insensitive
  /// filesystem, so the ordering rule below is only reachable from a name table built in a test.
  fn from_project(project: ArchiveProject, label: String) -> Self {
    let (entries, collisions, shadowed) = Self::index(&project);

    Self {
      collisions,
      entries,
      label,
      project,
      shadowed,
    }
  }

  /// Folds authored names to engine identities, ranking every copy of one identity and classifying what lost.
  ///
  /// **Last wins**, as `CLocatorAPI::Register` resolves it: it lower-cases a name before its name-table lookup and
  /// overwrites on a hit, so the later registration answers (`xray-16/src/xrCore/LocatorAPI.cpp`). Later here means the
  /// later volume in [`ArchiveProject`] order, so a patch volume overrides `Textures\A.DDS` exactly as it overrides
  /// `textures\a.dds`.
  ///
  /// A loser is an **override** when the copy ahead of it sits in another volume, and **unreachable** only when that
  /// copy sits in the same one — where no volume order separated them, which is what `XrayPathCollision` means by
  /// having no priority to appeal to. Within one volume the engine order is the header chunk order and the authored
  /// name stands in for it, which is why that arm is the one reported as an authoring error.
  ///
  /// Neither is refused: a person has to be able to open a volume set to learn what is wrong with it, and the engine
  /// does not refuse it either.
  // todo: Header order is dropped by the reader, so the within-volume rule is an approximation.
  fn index(
    project: &ArchiveProject,
  ) -> (
    HashMap<String, IndexedEntry>,
    Vec<XrayPathCollision>,
    Vec<XraySourceShadowedCopy>,
  ) {
    let mut entries: HashMap<String, IndexedEntry> = HashMap::with_capacity(project.files.len());
    // Losers only, so an uncontested identity - which is nearly all of them - allocates nothing here at all.
    let mut losers: Vec<(String, &ArchiveFileDescriptor)> = Vec::new();

    for (name, descriptor) in &project.files {
      if descriptor.is_directory {
        continue;
      }

      let Some(normalized) = Self::to_identity(descriptor) else {
        continue;
      };

      match entries.entry(normalized) {
        Entry::Vacant(slot) => {
          slot.insert(IndexedEntry::of(descriptor, name));
        }
        Entry::Occupied(mut slot) => {
          let incumbent: &ArchiveFileDescriptor = project
            .files
            .get(&slot.get().name)
            .expect("an indexed name belongs to the table it was read from");

          if Self::to_precedence(descriptor) > Self::to_precedence(incumbent) {
            losers.push((slot.key().clone(), incumbent));
            slot.insert(IndexedEntry::of(descriptor, name));
          } else {
            losers.push((slot.key().clone(), descriptor));
          }
        }
      }
    }

    // What the name-table merge overwrote. Each is outranked by the entry that displaced it - same authored name, later
    // volume - so one can never win its identity, and every one of them is a loser without being compared.
    for descriptor in &project.shadowed {
      if descriptor.is_directory {
        continue;
      }

      if let Some(normalized) = Self::to_identity(descriptor) {
        losers.push((normalized, descriptor));
      }
    }

    let (collisions, shadowed) = Self::classify(project, &entries, losers);

    (entries, collisions, shadowed)
  }

  /// Sorts each contested identity into the copies a patch buried and the copies nothing can reach.
  fn classify(
    project: &ArchiveProject,
    entries: &HashMap<String, IndexedEntry>,
    losers: Vec<(String, &ArchiveFileDescriptor)>,
  ) -> (Vec<XrayPathCollision>, Vec<XraySourceShadowedCopy>) {
    let mut collisions: Vec<XrayPathCollision> = Vec::new();
    let mut shadowed: Vec<XraySourceShadowedCopy> = Vec::new();
    let mut contested: HashMap<String, Vec<&ArchiveFileDescriptor>> = HashMap::new();

    for (logical_path, descriptor) in losers {
      contested.entry(logical_path).or_default().push(descriptor);
    }

    for (logical_path, mut behind) in contested {
      let Some(winner) = entries
        .get(&logical_path)
        .and_then(|entry| project.files.get(&entry.name))
      else {
        continue;
      };

      // Highest precedence first, so each copy is judged against the one directly ahead of it. The winner outranks
      // every one of them by construction, so it leads without being sorted in.
      behind.sort_by(|first, second| Self::to_precedence(second).cmp(&Self::to_precedence(first)));

      for (index, loser) in behind.iter().enumerate() {
        let ahead: &ArchiveFileDescriptor = if index == 0 { winner } else { behind[index - 1] };

        if ahead.volume == loser.volume {
          collisions.push(XrayPathCollision {
            kept: Self::to_site(project, ahead),
            logical_path: XrayLogicalPath::from_normalized(logical_path.clone()),
            unreachable: Self::to_site(project, loser),
          });
        } else {
          shadowed.push(XraySourceShadowedCopy {
            container: Self::to_volume_container(project, loser.volume),
            logical_path: logical_path.clone(),
            size: u64::from(loser.size_real),
          });
        }
      }
    }

    // Sorted, so a report reads the same twice: the groups above come out of a `HashMap` in no order at all.
    collisions.sort_by(|first, second| first.logical_path.as_str().cmp(second.logical_path.as_str()));
    shadowed.sort_by(|first, second| first.logical_path.cmp(&second.logical_path));

    (collisions, shadowed)
  }

  /// The engine identity one entry folds to, or nothing when its name is not one.
  fn to_identity(descriptor: &ArchiveFileDescriptor) -> Option<String> {
    normalize_logical(&descriptor.name)
      .inspect_err(|error| log::warn!("Skipping archive entry '{}': {error}", descriptor.name))
      .ok()
  }

  /// What decides between two copies of one identity: volume order first, then the authored name.
  ///
  /// Read off the descriptor rather than recovered by matching names back to the table, because a displaced copy is
  /// not in that table and shares its name with the entry that displaced it.
  fn to_precedence(descriptor: &ArchiveFileDescriptor) -> (u32, &str) {
    (descriptor.volume, &descriptor.name)
  }

  /// The volume one copy sits in, which is what a person needs to act on it.
  fn to_volume_container(project: &ArchiveProject, volume: u32) -> XrayAssetContainer {
    XrayAssetContainer::Archive {
      path: project
        .archives
        .get(volume as usize)
        .map(|volume| volume.path.clone())
        .unwrap_or_else(|| project.root.clone()),
    }
  }

  /// Where an authored entry sits, for a collision diagnostic.
  fn to_site(project: &ArchiveProject, descriptor: &ArchiveFileDescriptor) -> XrayCollisionSite {
    XrayCollisionSite::Archived {
      name: descriptor.name.to_string(),
      volume: project
        .archives
        .get(descriptor.volume as usize)
        .map(|volume| volume.path.clone())
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

  /// Names the volume the entry sits in, not the set's root: a directory-mounted set answers for many volumes, and a
  /// caller comparing two copies of one path needs to see which files they are.
  fn locate(&self, path: &str) -> Option<XrayAssetContainer> {
    self
      .entries
      .get(path)
      .map(|entry| Self::to_volume_container(&self.project, entry.volume))
  }

  fn read(&self, path: &str) -> XrfResult<Vec<u8>> {
    let Some(entry) = self.entries.get(path) else {
      // Absent, not unreadable: the distinction lets a caller fall back rather than fail.
      return Err(XrfError::new_not_found_error(format!(
        "no archive entry '{path}' in {}",
        self.label
      )));
    };

    self.project.read_file_bytes(&entry.name)
  }

  /// Answers from the volume's name table, so no entry is decompressed to learn its size.
  fn get_size(&self, path: &str) -> Option<u64> {
    self
      .entries
      .get(path)
      .and_then(|entry| self.project.files.get(&entry.name))
      .map(|descriptor| u64::from(descriptor.size_real))
  }

  /// Answers from the keyed name table, which already folded away directory records and unreachable duplicates.
  fn count_entries(&self) -> usize {
    self.entries.len()
  }

  /// The set's volumes as [`ArchiveProject`] merged them, so the last one is the one that won a shared name.
  fn list_volumes(&self) -> &[ArchiveDescriptor] {
    &self.project.archives
  }

  /// One per volume of the set, from each `[header] entry_point` the reader already stripped its alias from.
  fn list_declared_roots(&self) -> Vec<XrayDeclaredRoot> {
    self
      .project
      .archives
      .iter()
      .map(|volume| XrayDeclaredRoot {
        source: volume.path.clone(),
        root: volume.output_root_path.clone(),
      })
      .collect()
  }

  /// Answers from the volume's name table, which is where the packer recorded it and where the engine reads it back.
  fn get_recorded_crc(&self, path: &str) -> Option<u32> {
    self
      .entries
      .get(path)
      .and_then(|entry| self.project.files.get(&entry.name))
      .map(|descriptor| descriptor.crc)
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

  fn list_shadowed(&self) -> &[XraySourceShadowedCopy] {
    &self.shadowed
  }
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;
  use std::path::{Path, PathBuf};
  use std::sync::Arc;

  use xrf_archive::{ArchiveDescriptor, ArchiveFileDescriptor, ArchiveProject, ArchiveReadPolicy};

  use crate::{XrayAssetContainer, XrayAssetSource, XrayCollisionSite, XrayPathCollision, XraySourceShadowedCopy};

  use super::XrayArchiveSource;

  const BASE: &str = "C:\\game\\db\\base.db0";
  const PATCH: &str = "C:\\game\\db\\patch.db1";

  /// A merged name table, as [`ArchiveProject`] hands one over: volumes in merge order, entries keyed as authored.
  ///
  /// Built rather than packed because a case-only pair cannot exist in one directory on a case-insensitive filesystem,
  /// which is the only way to reach the within-volume half of the rule. The packed round trip is covered by
  /// `xrf-pack`'s `asset_source_tests`.
  fn project(volumes: &[&str], files: &[(&str, &str, u32)]) -> ArchiveProject {
    let merged: (HashMap<Arc<str>, ArchiveFileDescriptor>, Vec<ArchiveFileDescriptor>) = merge(volumes, files);

    ArchiveProject {
      archives: volumes
        .iter()
        .map(|volume| ArchiveDescriptor {
          created_at: None,
          entries: 0,
          modified_at: None,
          output_root_path: PathBuf::from("gamedata"),
          path: PathBuf::from(*volume),
          size_compressed: 0,
          size_real: 0,
        })
        .collect(),
      files: merged.0,
      read_policy: ArchiveReadPolicy::default(),
      root: PathBuf::from("C:\\game\\db"),
      shadowed: merged.1,
      size_real: 0,
    }
  }

  /// Merges the cases into a name table the way [`ArchiveProject`] does: volume by volume, keeping what is displaced.
  fn merge(
    volumes: &[&str],
    files: &[(&str, &str, u32)],
  ) -> (HashMap<Arc<str>, ArchiveFileDescriptor>, Vec<ArchiveFileDescriptor>) {
    let mut table: HashMap<Arc<str>, ArchiveFileDescriptor> = HashMap::new();
    let mut displaced: Vec<ArchiveFileDescriptor> = Vec::new();

    // Cases name their volume, which reads better than a position; the project addresses it by position, so the
    // fixture resolves one to the other exactly as reading a real set does.
    let mut ordered: Vec<(u32, &str, u32)> = files
      .iter()
      .map(|(volume, name, size)| {
        let index: u32 = volumes
          .iter()
          .position(|candidate| candidate == volume)
          .expect("an entry names a volume of its own set") as u32;

        (index, *name, *size)
      })
      .collect();

    // By volume, because a set is read in merge order however a case happens to list its entries.
    ordered.sort_by_key(|(index, _, _)| *index);

    for (index, name, size) in ordered {
      let name: Arc<str> = Arc::from(name);
      let descriptor: ArchiveFileDescriptor =
        ArchiveFileDescriptor::new(0, Arc::clone(&name), 0, size, size).in_volume(index);

      if let Some(previous) = table.insert(name, descriptor) {
        displaced.push(previous);
      }
    }

    (table, displaced)
  }

  fn source(volumes: &[&str], files: &[(&str, &str, u32)]) -> XrayArchiveSource {
    XrayArchiveSource::from_project(project(volumes, files), String::from("db"))
  }

  /// Asserts a container names the volume file a copy sits in, rather than the set's root.
  fn assert_container(container: &XrayAssetContainer, expected_volume: &str) {
    match container {
      XrayAssetContainer::Archive { path } => assert_eq!(path, Path::new(expected_volume)),
      XrayAssetContainer::Directory { root, .. } => panic!("archived copy expected, got loose {}", root.display()),
    }
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
  fn a_later_volume_overrides_a_case_folded_entry_rather_than_colliding() {
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

    assert!(
      source.get_collisions().is_empty(),
      "an override is not an authoring error"
    );

    let shadowed: &[XraySourceShadowedCopy] = source.list_shadowed();

    assert_eq!(shadowed.len(), 1);
    assert_eq!(shadowed[0].logical_path, "textures\\a.dds");
    assert_eq!(shadowed[0].size, 10, "the base copy, which is what the patch buried");
    assert_container(&shadowed[0].container, BASE);
  }

  #[test]
  fn a_later_volume_overrides_an_identically_spelled_entry() {
    // The ordinary patch, and the case that was invisible: the merged name table keeps one row, so without the
    // displaced descriptor nothing anywhere could say the base copy had ever existed.
    let source: XrayArchiveSource = source(
      &[BASE, PATCH],
      &[(BASE, "configs\\system.ltx", 10), (PATCH, "configs\\system.ltx", 20)],
    );

    assert_eq!(source.get_size("configs\\system.ltx"), Some(20));
    assert!(source.get_collisions().is_empty());

    let shadowed: &[XraySourceShadowedCopy] = source.list_shadowed();

    assert_eq!(shadowed.len(), 1);
    assert_eq!(shadowed[0].logical_path, "configs\\system.ltx");
    assert_eq!(shadowed[0].size, 10);
    assert_container(&shadowed[0].container, BASE);
  }

  #[test]
  fn a_copy_is_unreachable_only_behind_one_from_its_own_volume() {
    // Three copies of one identity: the patch wins, the base copy behind it is an override, and the second base copy
    // is unreachable - nothing orders it against the first but an authored name the reader no longer has.
    let source: XrayArchiveSource = source(
      &[BASE, PATCH],
      &[
        (BASE, "textures\\a.dds", 10),
        (BASE, "Textures\\A.DDS", 11),
        (PATCH, "textures\\a.dds", 20),
      ],
    );

    assert_eq!(source.get_size("textures\\a.dds"), Some(20));

    let shadowed: &[XraySourceShadowedCopy] = source.list_shadowed();

    assert_eq!(shadowed.len(), 1, "the base copy the patch buried");
    assert_eq!(shadowed[0].size, 10);

    let collisions: &[XrayPathCollision] = source.get_collisions();

    assert_eq!(collisions.len(), 1, "the pair authored twice inside the base volume");
    assert_site(&collisions[0].kept, BASE, "textures\\a.dds");
    assert_site(&collisions[0].unreachable, BASE, "Textures\\A.DDS");
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
      assert!(source.get_collisions().is_empty());
      assert_eq!(source.list_shadowed().len(), 1);
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
