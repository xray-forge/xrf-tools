use xrf_extension::XrayExtension;

/// Converts a sound reference into the name the engine registers it under. Behind
/// [`crate::XrayAssetType::sound_reference_name`].
pub(crate) fn sound_reference_name(reference: &str) -> String {
  let reference: String = reference.trim().replace('/', "\\").to_ascii_lowercase();
  let reference: &str = reference.strip_prefix("sounds\\").unwrap_or(&reference);

  if !XrayExtension::Ogg.matches(reference) {
    return reference.to_string();
  }

  // The extension matched, so the final dot is inside the final segment and this splits the name rather than a
  // directory holding one.
  reference
    .rsplit_once('.')
    .map_or(reference, |(stem, _)| stem)
    .to_string()
}

#[cfg(test)]
mod tests {
  use super::sound_reference_name;

  #[test]
  fn strips_the_implied_root_and_extension_from_a_name() {
    assert_eq!(
      sound_reference_name(" sounds/weapons/ak74_shot.ogg "),
      "weapons\\ak74_shot"
    );
    assert_eq!(sound_reference_name("Weapons\\AK74_Shot"), "weapons\\ak74_shot");
  }

  #[test]
  fn keeps_a_name_that_merely_ends_in_the_letters() {
    // The last dotted byte-suffix rule in the workspace lived here, and it was safe only because the value above it is
    // folded first. The splitter answers the same for a lower-case name and says why for the others.
    assert_eq!(sound_reference_name("weapons\\ak74_myogg"), "weapons\\ak74_myogg");
    assert_eq!(sound_reference_name("weapons\\ogg"), "weapons\\ogg");
  }

  #[test]
  fn leaves_a_directory_carrying_the_dot_alone() {
    // `rsplit_once` is reached only once the extension matched, so an unextensioned name inside `sounds.old\` keeps
    // every character of it rather than losing its last directory.
    assert_eq!(sound_reference_name("weapons.old\\ak74_shot"), "weapons.old\\ak74_shot");
  }
}
