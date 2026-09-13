use xrf_extension::XrayExtension;

use crate::discovery::dltx_attachment::DltxAttachment;

/// Depth assigned to the base root. Its includes count up from here.
pub const DLTX_BASE_DEPTH: i32 = 0;

/// Depth step between consecutive mod files.
///
/// Two hundred slots each so a mod file's own include chain, which counts up one per level, cannot reach the next mod
/// file's band. `Xr_ini.cpp`.
pub const DLTX_MOD_DEPTH_STEP: i32 = -200;

/// What a patch file's name opens with, the fixed half of the engine's `mod_<stem>_*.ltx` glob.
const DLTX_MOD_PREFIX: &str = "mod_";

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
    let Some(stem) = Self::config_stem(base_name) else {
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

    // `FS_FileSet` is ordered by name, so the engine's load order is ascending and every run agrees on it. Over folded
    // names, because that set holds the lower-cased names `FS_Path::_update` registered; sorting as recorded would put
    // `mod_System_b.ltx` before `mod_system_a.ltx` and hand the win to the wrong file.
    matched.sort_by_cached_key(|name| name.to_ascii_lowercase());

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
    Self::strip_prefix_folded(name, DLTX_MOD_PREFIX)
      .and_then(|rest| Self::strip_prefix_folded(rest, stem))
      .and_then(|rest| rest.strip_prefix('_'))
      .is_some_and(|rest| XrayExtension::Ltx.matches(rest))
  }

  /// Whether `name` is another base config whose stem extends `stem`, which makes mod names ambiguous between them.
  ///
  /// Requires at least one character after the separator, which is why `system_.ltx` would not make `system.ltx`
  /// ambiguous and neither does the base file itself.
  fn is_longer_base_than(name: &str, stem: &str) -> bool {
    Self::config_stem(name)
      .and_then(|other| Self::strip_prefix_folded(other, stem))
      .and_then(|rest| rest.strip_prefix('_'))
      .is_some_and(|rest| !rest.is_empty())
  }

  /// Whether `name` reads as a mod file of `longer_base` rather than of the shorter stem being resolved.
  ///
  /// The engine tests `mod_<longer stem>_.+\.ltx` and needs at least one character where the `.+` sits.
  fn belongs_to_longer_base(name: &str, longer_base: &str) -> bool {
    let Some(longer_stem) = Self::config_stem(longer_base) else {
      return false;
    };

    Self::strip_prefix_folded(name, DLTX_MOD_PREFIX)
      .and_then(|rest| Self::strip_prefix_folded(rest, longer_stem))
      .and_then(|rest| rest.strip_prefix('_'))
      .and_then(Self::config_stem)
      .is_some_and(|rest| !rest.is_empty())
  }

  /// `value` without `prefix`, compared without case.
  /// ASCII only, as the rest of the workspace's folding is; a Cyrillic config name compares as written.
  fn strip_prefix_folded<'a>(value: &'a str, prefix: &str) -> Option<&'a str> {
    value
      .get(..prefix.len())
      .filter(|start| start.eq_ignore_ascii_case(prefix))
      .map(|_| &value[prefix.len()..])
  }

  /// A config's name without its extension, or `None` when the name is not a config at all.
  fn config_stem(name: &str) -> Option<&str> {
    if !XrayExtension::Ltx.matches(name) {
      return None;
    }

    name.rsplit_once('.').map(|(stem, _)| stem)
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

  #[test]
  fn attaches_a_mod_file_whose_case_does_not_match_the_base() {
    // Every comparison here is folded, because the engine's are made against names it folded first. A listing hands
    // back the case a mod author typed, so this pairs a base and patches that agree on nothing but the letters - which
    // an earlier pass claimed to fix while hand-picking matching case, leaving the case that mattered still broken.
    assert_eq!(
      names(
        "System.LTX",
        &["System.LTX", "mod_SYSTEM_a.ltx", "MOD_system_b.Ltx", "mod_other_c.ltx"]
      ),
      vec!["mod_SYSTEM_a.ltx", "MOD_system_b.Ltx"]
    );
  }

  #[test]
  fn orders_folded_so_the_alphabetically_last_file_wins_whatever_case_it_is_in() {
    // Byte order puts every upper-case name first, which would hand the win to `mod_System_A.ltx`. The engine's set
    // holds folded names, so `b` outranks `A` there and has to here.
    let attachments: Vec<DltxAttachment> = DltxDiscovery::attachments_of(
      "system.ltx",
      &["mod_system_b.ltx", "mod_System_A.ltx"]
        .iter()
        .map(|it| String::from(*it))
        .collect::<Vec<String>>(),
    );

    assert_eq!(attachments[0].name, "mod_System_A.ltx");
    assert_eq!(attachments[1].name, "mod_system_b.ltx");
    assert!(attachments[1].depth < attachments[0].depth);
  }

  #[test]
  fn resolves_ambiguity_whatever_case_the_longer_base_was_recorded_in() {
    // The ambiguity rule folds too, or a longer base spelled differently stops claiming the files that belong to it
    // and both configs load them.
    let siblings: &[&str] = &[
      "system.ltx",
      "System_Foo.LTX",
      "mod_system_bar.ltx",
      "mod_SYSTEM_foo_bar.ltx",
    ];

    assert_eq!(names("system.ltx", siblings), vec!["mod_system_bar.ltx"]);
    assert_eq!(names("System_Foo.LTX", siblings), vec!["mod_SYSTEM_foo_bar.ltx"]);
  }

  #[test]
  fn a_name_merely_ending_in_the_letters_is_not_a_config() {
    // What the dot in the suffix form was doing, now the splitter's job: `systemxltx` is one name, not a config.
    assert_eq!(names("systemxltx", &["mod_systemxltx_a.ltx"]), Vec::<String>::new());
    assert_eq!(names("system.ltx", &["mod_system_altx"]), Vec::<String>::new());
  }
}
