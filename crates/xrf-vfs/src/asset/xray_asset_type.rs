use serde::{Deserialize, Serialize};

/// Asset category inferred from an X-Ray logical path's extension or recognized suffix.
///
/// Serialized so a consumer can name the kind it wants without the crate growing a command per kind, which is the same
/// reason [`XrayAssetType::get_rules`] is a table rather than a method each.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum XrayAssetType {
  Ai,
  Anm,
  CForm,
  Dds,
  Dm,
  Efd,
  EnvMod,
  FogVol,
  Game,
  Geom,
  GeomX,
  Hom,
  Ini,
  Level,
  Lights,
  Ltx,
  Misc,
  Ogf,
  Ogg,
  Ogm,
  Omf,
  Ppe,
  PsStatic,
  SndStatic,
  Script,
  Seq,
  Shader,
  Spawn,
  Thm,
  Wallmarks,
  Details,
  XrPack,
}

/// Where a kind of asset lives and what extension the engine loads it as.
///
/// A table rather than a method per kind, so resolving a new kind is a row here instead of a new accessor on every
/// resolver — which is how the same extension rule came to be written twice and drift.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct XrayAssetRules {
  /// Logical directory the engine resolves this kind under.
  pub directory: &'static str,
  /// Extension the engine loads.
  pub extension: &'static str,
  /// Extensions a reference may be authored with, which resolve to [`Self::extension`].
  pub authoring_extensions: &'static [&'static str],
}

impl XrayAssetType {
  /// The X-Ray shader library's fixed gamedata-relative logical path.
  ///
  /// A constant rather than a [`XrayAssetRules`] row, because [`Self::Shader`] has no directory-plus-extension home —
  /// the library is one fixed file beside the trees the rules describe.
  pub const SHADER_LIBRARY_PATH: &'static str = super::shader::SHADER_LIBRARY_LOGICAL_PATH;

  /// Converts a sound reference into the name the engine registers it under.
  ///
  /// Both the `sounds` root and the `.ogg` extension are implied, and a config may spell either out. Case and separators
  /// are normalized too, so a reference matches a registered name whichever way it was written. Sounds get a dedicated
  /// conversion where other kinds go through [`Self::get_rules`], because the registered name strips the extension
  /// instead of gaining one.
  pub fn sound_reference_name(reference: &str) -> String {
    super::sound::sound_reference_name(reference)
  }

  /// Where this kind lives, for the kinds with one canonical home.
  ///
  /// `None` covers two different cases, and both are deliberate rather than gaps to fill in speculatively: a kind whose
  /// home is not one directory — `Level` names a directory per level, `Shader` loads a dozen extensions — and a kind no
  /// caller resolves by reference yet. Add a row when a consumer needs one, with evidence from a real tree.
  pub fn get_rules(self) -> Option<XrayAssetRules> {
    let (directory, extension, authoring_extensions): (&str, &str, &[&str]) = match self {
      Self::Ogf => ("meshes", ".ogf", &[]),
      Self::Omf => ("meshes", ".omf", &[]),
      // A renderer reference may name the authored source; the engine loads the compiled `.dds` beside it.
      Self::Dds => ("textures", ".dds", &["tga", "bmp", "ogm"]),
      Self::Thm => ("textures", ".thm", &[]),
      Self::Ogg => ("sounds", ".ogg", &[]),
      Self::Ltx => ("configs", ".ltx", &[]),
      Self::Script => ("scripts", ".script", &[]),
      Self::Ppe => ("anims", ".ppe", &[]),
      _ => return None,
    };

    Some(XrayAssetRules {
      authoring_extensions,
      directory,
      extension,
    })
  }

  pub(crate) fn from_logical_path(path: &str) -> Option<Self> {
    match path.rsplit_once('.').map(|(_, extension)| extension) {
      Some("ai") => Some(Self::Ai),
      Some("anm" | "anm1") => Some(Self::Anm),
      Some("cform") => Some(Self::CForm),
      Some("dds") => Some(Self::Dds),
      Some("details") => Some(Self::Details),
      Some("dm") => Some(Self::Dm),
      Some("efd") => Some(Self::Efd),
      Some("env_mod") => Some(Self::EnvMod),
      Some("fog_vol") => Some(Self::FogVol),
      Some("game") => Some(Self::Game),
      Some("geom") => Some(Self::Geom),
      Some("geomx") => Some(Self::GeomX),
      Some("hom") => Some(Self::Hom),
      Some("ini") => Some(Self::Ini),
      Some("lights") => Some(Self::Lights),
      Some("log" | "bat" | "py" | "cmd") => Some(Self::Misc),
      Some("ltx") => Some(Self::Ltx),
      Some("ogf") => Some(Self::Ogf),
      Some("ogg") => Some(Self::Ogg),
      Some("ogm") => Some(Self::Ogm),
      Some("omf") => Some(Self::Omf),
      Some("ppe") => Some(Self::Ppe),
      Some("ps" | "s" | "s_" | "h" | "vs" | "cs" | "hs" | "ds" | "gs") => Some(Self::Shader),
      Some("ps_static") => Some(Self::PsStatic),
      Some("script") => Some(Self::Script),
      Some("seq" | "seq_") => Some(Self::Seq),
      Some("snd_static") => Some(Self::SndStatic),
      Some("spawn") => Some(Self::Spawn),
      Some("thm") => Some(Self::Thm),
      Some("wallmarks") => Some(Self::Wallmarks),
      Some("xr") => Some(Self::XrPack),
      // Reached only by a path carrying no `.` at all, which is how a level bundle's `level` file is named.
      _ if path.ends_with("level") => Some(Self::Level),
      _ => None,
    }
  }
}

impl XrayAssetRules {
  /// Converts a raw engine reference into the logical path below the kind's directory.
  ///
  /// An authoring extension is replaced rather than appended, and a reference already carrying the loaded extension is
  /// left alone. Both comparisons ignore case, because a reference authored as `wpn\wpn_ak74.OGF` names the same asset.
  pub fn to_logical_path(&self, reference: &str) -> String {
    if let Some((stem, extension)) = reference.rsplit_once('.') {
      if self
        .authoring_extensions
        .iter()
        .any(|known| extension.eq_ignore_ascii_case(known))
      {
        return format!("{stem}{}", self.extension);
      }

      if extension.eq_ignore_ascii_case(self.extension.trim_start_matches('.')) {
        return reference.to_string();
      }
    }

    format!("{reference}{}", self.extension)
  }
}

#[cfg(test)]
mod tests {
  use super::{XrayAssetRules, XrayAssetType};

  fn rules(asset_type: XrayAssetType) -> XrayAssetRules {
    asset_type.get_rules().expect("kind has a canonical home")
  }

  #[test]
  fn replaces_an_authoring_extension_with_the_loaded_one() {
    assert_eq!(
      rules(XrayAssetType::Dds).to_logical_path("pfx\\smoke.tga"),
      "pfx\\smoke.dds"
    );
    assert_eq!(
      rules(XrayAssetType::Dds).to_logical_path("pfx\\smoke.TGA"),
      "pfx\\smoke.dds"
    );
    assert_eq!(
      rules(XrayAssetType::Dds).to_logical_path("pfx\\smoke.bmp"),
      "pfx\\smoke.dds"
    );
  }

  #[test]
  fn appends_the_extension_when_a_reference_omits_it() {
    assert_eq!(
      rules(XrayAssetType::Dds).to_logical_path("pfx\\smoke"),
      "pfx\\smoke.dds"
    );
    assert_eq!(
      rules(XrayAssetType::Ogg).to_logical_path("weapons\\ak74_shot"),
      "weapons\\ak74_shot.ogg"
    );
    assert_eq!(
      rules(XrayAssetType::Ogg).to_logical_path("weapons\\ak74_shot.OGG"),
      "weapons\\ak74_shot.OGG",
      "an uppercase extension is not doubled"
    );
    assert_eq!(
      rules(XrayAssetType::Ogf).to_logical_path("actors\\stalker"),
      "actors\\stalker.ogf"
    );
  }

  #[test]
  fn leaves_an_already_loaded_extension_alone_whatever_its_case() {
    // Appending a second extension resolves to nothing, which is how mesh references silently failed.
    assert_eq!(
      rules(XrayAssetType::Ogf).to_logical_path("actors\\stalker.OGF"),
      "actors\\stalker.OGF"
    );
    assert_eq!(
      rules(XrayAssetType::Dds).to_logical_path("pfx\\smoke.dds"),
      "pfx\\smoke.dds"
    );
  }

  #[test]
  fn treats_an_unknown_extension_as_part_of_the_name() {
    // A reference is not a filename: `smoke.png` names an asset the engine loads as `smoke.png.dds`.
    assert_eq!(
      rules(XrayAssetType::Dds).to_logical_path("pfx\\smoke.png"),
      "pfx\\smoke.png.dds"
    );
  }

  #[test]
  fn answers_no_rules_for_kinds_without_one_home() {
    // `Level` names a directory per level and `Shader` loads a dozen extensions; neither is a directory-plus-extension pair.
    assert!(XrayAssetType::Level.get_rules().is_none());
    assert!(XrayAssetType::Shader.get_rules().is_none());
  }

  #[test]
  fn places_each_known_kind_under_its_engine_directory() {
    assert_eq!(rules(XrayAssetType::Ogf).directory, "meshes");
    assert_eq!(rules(XrayAssetType::Thm).directory, "textures");
    assert_eq!(rules(XrayAssetType::Ogg).directory, "sounds");
    assert_eq!(rules(XrayAssetType::Ltx).directory, "configs");
    assert_eq!(rules(XrayAssetType::Script).directory, "scripts");
    assert_eq!(rules(XrayAssetType::Ppe).directory, "anims");
  }

  #[test]
  fn reads_a_kind_off_an_extension() {
    assert_eq!(
      XrayAssetType::from_logical_path("shaders\\lod.s"),
      Some(XrayAssetType::Shader)
    );
    assert_eq!(
      XrayAssetType::from_logical_path("shaders\\model.ps"),
      Some(XrayAssetType::Shader)
    );
    assert_eq!(
      XrayAssetType::from_logical_path("textures\\wpn\\ak74.dds"),
      Some(XrayAssetType::Dds)
    );
    // A level bundle's own file carries no extension, which is the only way the trailing guard is reached.
    assert_eq!(
      XrayAssetType::from_logical_path("levels\\l01_escape\\level"),
      Some(XrayAssetType::Level)
    );
    assert_eq!(XrayAssetType::from_logical_path("readme"), None);
  }
}
