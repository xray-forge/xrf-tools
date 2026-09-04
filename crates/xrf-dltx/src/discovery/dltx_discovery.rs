use crate::discovery::dltx_attachment::DltxAttachment;

/// Depth assigned to the base root. Its includes count up from here.
pub const DLTX_BASE_DEPTH: i32 = 0;

/// Depth step between consecutive mod files.
///
/// Two hundred slots each so a mod file's own include chain, which counts up one per level, cannot reach the next mod
/// file's band. `Xr_ini.cpp`.
pub const DLTX_MOD_DEPTH_STEP: i32 = -200;

/// Works out which files patch a base config, in the order the engine applies them.
pub struct DltxDiscovery;

impl DltxDiscovery {
  /// The mod files attached to `base_name`, ordered as the engine loads them.
  ///
  /// `siblings` is every file name in the base file's own directory; nothing below it is searched. A candidate that
  /// belongs to a longer base name is left for that base, which is the ambiguity rule: with `system.ltx` and
  /// `system_foo.ltx` both present, `mod_system_foo_bar.ltx` patches only the latter.
  ///
  /// The alphabetically last file wins a conflict, because depth decreases as the list advances and lower depth beats
  /// higher (`Xr_ini.cpp`).
  pub fn attachments_of(base_name: &str, siblings: &[String]) -> Vec<DltxAttachment> {
    let Some(stem) = base_name.strip_suffix(".ltx") else {
      return Vec::new();
    };

    let ambiguous: Vec<&String> = siblings
      .iter()
      .filter(|name| Self::is_longer_base_than(name, stem))
      .collect();

    let mut matched: Vec<&String> = siblings
      .iter()
      .filter(|name| Self::is_mod_of(name, stem))
      .filter(|name| {
        !ambiguous
          .iter()
          .any(|longer| Self::belongs_to_longer_base(name, longer))
      })
      .collect();

    // `FS_FileSet` is ordered by name, so the engine's load order is ascending and every run agrees on it.
    matched.sort();

    matched
      .into_iter()
      .enumerate()
      .map(|(index, name)| DltxAttachment {
        depth: DLTX_MOD_DEPTH_STEP * (index as i32 + 1),
        name: name.clone(),
      })
      .collect()
  }

  /// Whether `name` is `mod_<stem>_*.ltx`.
  ///
  /// The trailing part may be empty, because the engine's glob lets `*` match nothing, so `mod_system_.ltx` is a mod
  /// file of `system.ltx` (`LocatorAPI_defs.cpp:117-140`).
  fn is_mod_of(name: &str, stem: &str) -> bool {
    name
      .strip_prefix("mod_")
      .and_then(|rest| rest.strip_prefix(stem))
      .and_then(|rest| rest.strip_prefix('_'))
      .is_some_and(|rest| rest.ends_with(".ltx"))
  }

  /// Whether `name` is another base config whose stem extends `stem`, which makes mod names ambiguous between them.
  ///
  /// Requires at least one character after the separator, which is why `system_.ltx` would not make `system.ltx`
  /// ambiguous and neither does the base file itself.
  fn is_longer_base_than(name: &str, stem: &str) -> bool {
    name
      .strip_suffix(".ltx")
      .and_then(|other| other.strip_prefix(stem))
      .and_then(|rest| rest.strip_prefix('_'))
      .is_some_and(|rest| !rest.is_empty())
  }

  /// Whether `name` reads as a mod file of `longer_base` rather than of the shorter stem being resolved.
  ///
  /// The engine tests `mod_<longer stem>_.+\.ltx` and needs at least one character where the `.+` sits.
  fn belongs_to_longer_base(name: &str, longer_base: &str) -> bool {
    let Some(longer_stem) = longer_base.strip_suffix(".ltx") else {
      return false;
    };

    name
      .strip_prefix("mod_")
      .and_then(|rest| rest.strip_prefix(longer_stem))
      .and_then(|rest| rest.strip_prefix('_'))
      .and_then(|rest| rest.strip_suffix(".ltx"))
      .is_some_and(|rest| !rest.is_empty())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn names(of: &str, siblings: &[&str]) -> Vec<String> {
    DltxDiscovery::attachments_of(
      of,
      &siblings.iter().map(|it| String::from(*it)).collect::<Vec<String>>(),
    )
    .into_iter()
    .map(|attachment| attachment.name)
    .collect()
  }

  #[test]
  fn matches_mod_files_of_the_base_stem() {
    assert_eq!(
      names(
        "system.ltx",
        &["system.ltx", "mod_system_a.ltx", "mod_system_b.ltx", "other.ltx"]
      ),
      vec!["mod_system_a.ltx", "mod_system_b.ltx"]
    );
  }

  #[test]
  fn orders_matches_ascending_so_the_last_one_wins() {
    let attachments: Vec<DltxAttachment> = DltxDiscovery::attachments_of(
      "system.ltx",
      &["mod_system_zzz.ltx", "mod_system_aaa.ltx"]
        .iter()
        .map(|it| String::from(*it))
        .collect::<Vec<String>>(),
    );

    assert_eq!(attachments[0].name, "mod_system_aaa.ltx");
    assert_eq!(attachments[0].depth, -200);
    assert_eq!(attachments[1].name, "mod_system_zzz.ltx");
    assert_eq!(attachments[1].depth, -400);

    // Lower depth wins, so the alphabetically last file outranks the first.
    assert!(attachments[1].depth < attachments[0].depth);
  }

  #[test]
  fn an_empty_suffix_still_matches() {
    // The engine's `*` may match nothing.
    assert_eq!(names("system.ltx", &["mod_system_.ltx"]), vec!["mod_system_.ltx"]);
  }

  #[test]
  fn a_mod_file_of_a_longer_base_is_left_to_that_base() {
    // `mod_system_foo_bar.ltx` reads as a mod of either `system` or `system_foo`; the longer base claims it.
    let siblings: &[&str] = &[
      "system.ltx",
      "system_foo.ltx",
      "mod_system_bar.ltx",
      "mod_system_foo_bar.ltx",
    ];

    assert_eq!(names("system.ltx", siblings), vec!["mod_system_bar.ltx"]);
    assert_eq!(names("system_foo.ltx", siblings), vec!["mod_system_foo_bar.ltx"]);
  }

  #[test]
  fn ambiguity_needs_the_longer_base_to_exist() {
    // Without `system_foo.ltx` present, nothing claims the file and the shorter base keeps it.
    assert_eq!(
      names("system.ltx", &["system.ltx", "mod_system_foo_bar.ltx"]),
      vec!["mod_system_foo_bar.ltx"]
    );
  }

  #[test]
  fn a_base_named_with_a_trailing_separator_does_not_create_ambiguity() {
    // `system_.ltx` has nothing after the separator, so the engine's `.+` never matches it.
    assert_eq!(
      names("system.ltx", &["system.ltx", "system_.ltx", "mod_system_a.ltx"]),
      vec!["mod_system_a.ltx"]
    );
  }

  #[test]
  fn ignores_files_that_are_not_mods_of_this_base() {
    assert_eq!(
      names(
        "system.ltx",
        &["mod_other_a.ltx", "mod_systema.ltx", "mod_system_a.xml", "system.ltx"]
      ),
      Vec::<String>::new()
    );
  }

  #[test]
  fn a_base_without_the_ltx_extension_has_no_attachments() {
    assert_eq!(names("system", &["mod_system_a.ltx"]), Vec::<String>::new());
  }
}
