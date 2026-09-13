/// The extension of an engine or host file name, without its dot.
///
/// X-Ray's own rule rather than `Path::extension`'s, on one point that matters: a name beginning with a dot is an
/// extension here, not a hidden file. The engine ships `shaders\r1\.s` and `shaders\r2\.s` — Lua scripts it loads
/// like any other shader — and `Path::extension` answers `None` for both, which had a viewer refusing to show them.
/// Nothing in game data is a Unix dotfile.
///
/// Returned as authored. A name table records a name as it was written, so folding case is the caller's decision.
pub fn get_file_extension(name: &str) -> Option<&str> {
  let segment: &str = match name.rfind(['\\', '/']) {
    Some(separator) => &name[separator + 1..],
    None => name,
  };

  segment.rsplit_once('.').map(|(_, extension)| extension)
}

/// Whether `name` carries `extension`, compared without case.
///
/// `extension` is undotted, the spelling [`get_file_extension`] answers with, and it is compared against that answer
/// rather than against the end of the name. The workspace had both rules at once: a logical path matched `".xml"` as a
/// byte suffix, while a name table matched `"xml"` against the split extension. The suffix form needed its dot only to
/// keep `notes.myxml` from passing as XML — which is the splitter's job anyway — and it disagreed with the splitter
/// about the engine's own leading-dot names, refusing a name that *is* its extension because it required a character
/// before the dot. One rule, and it is the splitter's.
pub fn has_extension(name: &str, extension: &str) -> bool {
  get_file_extension(name).is_some_and(|found| found.eq_ignore_ascii_case(extension))
}

#[cfg(test)]
mod tests {
  use super::{get_file_extension, has_extension};

  #[test]
  fn an_extension_is_whatever_follows_the_last_dot_of_the_last_segment() {
    assert_eq!(get_file_extension("configs\\system.ltx"), Some("ltx"));
    assert_eq!(get_file_extension("configs/system.ltx"), Some("ltx"));
    assert_eq!(get_file_extension("system.ltx"), Some("ltx"));
    assert_eq!(get_file_extension("meshes\\actor.anm1"), Some("anm1"));
    // Folded by whoever compares it, never here: a name table records a name as it was authored.
    assert_eq!(get_file_extension("TEXTURES\\A.DDS"), Some("DDS"));
  }

  #[test]
  fn a_leading_dot_names_an_extension_because_the_engine_ships_one() {
    // `shaders\r1\.s` and `shaders\r2\.s` are Lua scripts the engine loads. `Path::extension` calls both hidden files
    // and answers `None`, which is a Unix rule game data has no instance of.
    assert_eq!(get_file_extension("shaders\\r1\\.s"), Some("s"));
    assert_eq!(get_file_extension(".s"), Some("s"));
  }

  #[test]
  fn a_dot_in_a_directory_name_is_not_the_extension_of_an_extensionless_file() {
    assert_eq!(get_file_extension("configs\\weapons.old\\readme"), None);
    assert_eq!(get_file_extension("gamedata\\spawns"), None);
  }

  #[test]
  fn an_empty_extension_is_what_a_trailing_dot_leaves() {
    // Not a name anything ships, but the splitter has to answer something rather than panic on the slice.
    assert_eq!(get_file_extension("readme."), Some(""));
    assert!(!has_extension("readme.", "ltx"));
  }

  #[test]
  fn compares_an_extension_without_case() {
    assert!(has_extension("configs\\system.ltx", "ltx"));
    assert!(has_extension("configs\\SYSTEM.LTX", "ltx"));
    assert!(has_extension("configs\\system.ltx", "LTX"));
  }

  #[test]
  fn refuses_a_name_that_merely_ends_with_the_extension() {
    // What the dot in the retired suffix form was there for. The splitter closes it without one.
    assert!(!has_extension("notes.myxml", "xml"));
    assert!(!has_extension("xml", "xml"));
    assert!(!has_extension("configs\\myltx", "ltx"));
  }

  #[test]
  fn a_name_that_is_its_own_extension_carries_it() {
    // The reversal this unified rule makes deliberately: the retired suffix form answered `false` here, while the
    // splitter beside it answered `Some("s")` for the same name. `shaders\r1\.s` is a file the engine loads.
    assert!(has_extension(".s", "s"));
    assert!(has_extension("shaders\\r1\\.s", "s"));
  }
}
