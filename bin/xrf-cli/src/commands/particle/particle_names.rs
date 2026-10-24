/// How the particle commands compare names a person types with names a library holds, alike in every command.
pub struct ParticleNames;

impl ParticleNames {
  /// A name as the engine compares it: in any case, with either slash.
  pub fn name_key(name: &str) -> String {
    name.trim().replace('/', "\\").to_lowercase()
  }

  /// A texture as the engine finds it: a name compared as [`Self::name_key`] does, with any extension dropped, as the
  /// texture loader drops one a sprite names (`pfx\pfx_ani-fire01.bmp`).
  pub fn texture_key(texture: &str) -> String {
    let key: String = Self::name_key(texture);

    match key.rfind('.') {
      Some(at) if !key[at..].contains('\\') => key[..at].to_owned(),
      _ => key,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::ParticleNames;

  #[test]
  fn keys_a_name_in_any_case_with_either_slash() {
    assert_eq!(ParticleNames::name_key(" Explosions/Campfire "), r"explosions\campfire");
  }

  #[test]
  fn keys_a_texture_without_its_extension_but_keeps_a_dot_in_a_folder() {
    assert_eq!(
      ParticleNames::texture_key("PFX/pfx_ani-fire01.bmp"),
      r"pfx\pfx_ani-fire01"
    );
    assert_eq!(ParticleNames::texture_key(r"pfx.old\fire"), r"pfx.old\fire");
  }
}
