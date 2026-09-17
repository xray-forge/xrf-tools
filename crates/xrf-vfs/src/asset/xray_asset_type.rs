use serde::{Deserialize, Serialize};
use xrf_extension::{XrayExtension, XrayExtensionOf};

use crate::path::XrayLogicalPath;

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
  Som,
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
  pub extension: XrayExtension,
  /// Extensions a reference may be authored with, which resolve to [`Self::extension`].
  pub authoring_extensions: &'static [XrayExtension],
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
  /// `None` covers two different cases, and both are intentional rather than gaps to fill in speculatively: a kind whose
  /// home is not one directory — `Level` names a directory per level, `Shader` loads a dozen extensions — and a kind no
  /// caller resolves by reference yet. Add a row when a consumer needs one, with evidence from a real tree.
  pub fn get_rules(self) -> Option<XrayAssetRules> {
    let (directory, extension, authoring_extensions): (&str, XrayExtension, &[XrayExtension]) = match self {
      Self::Ogf => ("meshes", XrayExtension::Ogf, &[]),
      Self::Omf => ("meshes", XrayExtension::Omf, &[]),
      // A renderer reference may name the authored source; the engine loads the compiled `.dds` beside it.
      Self::Dds => (
        "textures",
        XrayExtension::Dds,
        &[XrayExtension::Tga, XrayExtension::Bmp, XrayExtension::Ogm],
      ),
      Self::Thm => ("textures", XrayExtension::Thm, &[]),
      Self::Ogg => ("sounds", XrayExtension::Ogg, &[]),
      Self::Ltx => ("configs", XrayExtension::Ltx, &[]),
      Self::Script => ("scripts", XrayExtension::Script, &[]),
      Self::Ppe => ("anims", XrayExtension::Ppe, &[]),
      _ => return None,
    };

    Some(XrayAssetRules {
      authoring_extensions,
      directory,
      extension,
    })
  }

  /// The kind an extension names, or `None` for one that names no asset this models.
  ///
  /// Where an extension stops being a spelling and becomes engine-tree knowledge. The vocabulary knows that `anm` and
  /// `anm1` are two different files; only here is it known that the engine loads both as animations, that eleven
  /// spellings are all shader sources, and that `log` and `bat` are not assets so much as things found beside them.
  pub fn of(extension: XrayExtension) -> Option<Self> {
    Some(match extension {
      XrayExtension::Ai => Self::Ai,
      // Two spellings of one animation container, loaded the same way.
      XrayExtension::Anm | XrayExtension::Anm1 => Self::Anm,
      XrayExtension::CForm => Self::CForm,
      XrayExtension::Dds => Self::Dds,
      XrayExtension::Details => Self::Details,
      XrayExtension::Dm => Self::Dm,
      XrayExtension::Efd => Self::Efd,
      XrayExtension::EnvMod => Self::EnvMod,
      XrayExtension::FogVol => Self::FogVol,
      XrayExtension::Game => Self::Game,
      XrayExtension::Geom => Self::Geom,
      XrayExtension::GeomX => Self::GeomX,
      XrayExtension::Hom => Self::Hom,
      XrayExtension::Ini => Self::Ini,
      XrayExtension::Lights => Self::Lights,
      // Not assets: the logs, scripts and batch files a tree accumulates beside them, kept in one bucket because no
      // consumer resolves one by reference.
      XrayExtension::Log | XrayExtension::Bat | XrayExtension::Py | XrayExtension::Cmd => Self::Misc,
      XrayExtension::Ltx => Self::Ltx,
      XrayExtension::Ogf => Self::Ogf,
      XrayExtension::Ogg => Self::Ogg,
      XrayExtension::Ogm => Self::Ogm,
      XrayExtension::Omf => Self::Omf,
      XrayExtension::Ppe => Self::Ppe,
      XrayExtension::Ps
      | XrayExtension::S
      | XrayExtension::S_
      | XrayExtension::H
      | XrayExtension::Vs
      | XrayExtension::Cs
      | XrayExtension::Hs
      | XrayExtension::Ds
      | XrayExtension::Gs
      | XrayExtension::Hlsl
      | XrayExtension::Lua => Self::Shader,
      XrayExtension::PsStatic => Self::PsStatic,
      XrayExtension::Script => Self::Script,
      XrayExtension::Seq | XrayExtension::Seq_ => Self::Seq,
      XrayExtension::SndStatic => Self::SndStatic,
      XrayExtension::Som => Self::Som,
      XrayExtension::Spawn => Self::Spawn,
      XrayExtension::Thm => Self::Thm,
      XrayExtension::Wallmarks => Self::Wallmarks,
      XrayExtension::Xr => Self::XrPack,
      XrayExtension::Bmp
      | XrayExtension::Tga
      | XrayExtension::Png
      | XrayExtension::Jpg
      | XrayExtension::Jpeg
      | XrayExtension::Wav
      | XrayExtension::Htm
      | XrayExtension::Html
      | XrayExtension::Json
      | XrayExtension::Md
      | XrayExtension::Ts
      | XrayExtension::Xml => return None,
    })
  }

  /// The kind a logical path names, read from its extension and then from its name.
  pub fn from_logical_path(path: &str) -> Option<Self> {
    if let Some(asset_type) = XrayExtensionOf::of(path).known().and_then(Self::of) {
      return Some(asset_type);
    }

    // Reached only by a path carrying no extension this maps to, which is how a level bundle's `level` file is named.
    path.ends_with("level").then_some(Self::Level)
  }
}

impl XrayAssetRules {
  /// Converts a raw engine reference into the logical path below the kind's directory.
  ///
  /// An authoring extension is replaced rather than appended, and a reference already carrying the loaded extension is
  /// left alone. Both comparisons ignore case, because a reference authored as `wpn\wpn_ak74.OGF` names the same asset.
  pub fn to_logical_path(&self, reference: &str) -> String {
    if let Some((stem, extension)) = reference.rsplit_once('.') {
      if XrayExtension::parse(extension).is_some_and(|authored| self.authoring_extensions.contains(&authored)) {
        return format!("{stem}.{}", self.extension);
      }

      if self.extension.matches(reference) {
        return reference.to_string();
      }
    }

    format!("{reference}.{}", self.extension)
  }

  /// Converts a logical path below this kind's directory back into the reference the engine names it by.
  pub fn to_reference(&self, logical_path: &XrayLogicalPath) -> Option<String> {
    let below: &str = logical_path.strip_prefix(self.directory).ok()??;

    if !self.extension.matches(below) {
      return None;
    }

    // The extension matched, so the final dot is inside the final segment and this splits the name rather than a
    // directory holding one.
    let (reference, _) = below.rsplit_once('.')?;

    (!reference.is_empty()).then(|| reference.to_owned())
  }
}

#[cfg(test)]
mod tests {
  use xrf_extension::XrayExtension;

  use super::{XrayAssetRules, XrayAssetType};
  use crate::path::XrayLogicalPath;

  fn rules(asset_type: XrayAssetType) -> XrayAssetRules {
    asset_type.get_rules().expect("kind has a canonical home")
  }

  fn logical(path: &str) -> XrayLogicalPath {
    XrayLogicalPath::new(path).expect("valid logical path")
  }

  #[test]
  fn reads_a_reference_back_off_a_logical_path_in_the_kinds_home() {
    assert_eq!(
      rules(XrayAssetType::Dds)
        .to_reference(&logical("textures\\pfx\\smoke.dds"))
        .as_deref(),
      Some("pfx\\smoke")
    );
    assert_eq!(
      rules(XrayAssetType::Ogf)
        .to_reference(&logical("meshes\\actors\\stalker.ogf"))
        .as_deref(),
      Some("actors\\stalker")
    );
    // Round trip: the reference resolves back to the path it was read from.
    assert_eq!(
      rules(XrayAssetType::Dds).to_logical_path("pfx\\smoke"),
      "pfx\\smoke.dds"
    );
  }

  #[test]
  fn names_no_reference_for_a_path_outside_the_kinds_home_or_extension() {
    assert_eq!(
      rules(XrayAssetType::Dds).to_reference(&logical("levels\\l01\\lmap#0_1.dds")),
      None,
      "the directory has to match, not only the extension"
    );
    assert_eq!(
      rules(XrayAssetType::Dds).to_reference(&logical("textures_old\\a.dds")),
      None,
      "a directory sharing the prefix is not the home"
    );
    assert_eq!(
      rules(XrayAssetType::Dds).to_reference(&logical("textures\\pfx\\smoke.thm")),
      None,
      "a descriptor is not a texture, whatever directory it sits in"
    );
    assert_eq!(
      rules(XrayAssetType::Dds).to_reference(&logical("textures\\.dds")),
      None,
      "an extension alone names nothing"
    );
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
    // Not a gap in the authoring list: a reference is a name rather than a filename, so an extension the row does not
    // claim stays part of it and the loaded one is appended.
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
  fn names_the_loaded_extension_undotted_so_no_caller_has_to_trim_one() {
    // The dot used to be part of the stored spelling, which made every consumer strip it back off before joining it
    // to a host path - and `Path::with_extension` refuses a dotted one outright.
    assert_eq!(rules(XrayAssetType::Dds).extension, XrayExtension::Dds);
    assert_eq!(rules(XrayAssetType::Dds).extension.as_str(), "dds");
    assert_eq!(
      rules(XrayAssetType::Dds).authoring_extensions,
      &[XrayExtension::Tga, XrayExtension::Bmp, XrayExtension::Ogm]
    );
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

  #[test]
  fn gathers_the_spellings_the_engine_loads_the_same_way() {
    // The groupings are this crate's knowledge, not the vocabulary's: `anm1` is a different file from `anm`, and a
    // reader outside a mounted world is free to treat them differently.
    for extension in [XrayExtension::Anm, XrayExtension::Anm1] {
      assert_eq!(XrayAssetType::of(extension), Some(XrayAssetType::Anm));
    }

    for extension in [
      XrayExtension::Ps,
      XrayExtension::S,
      XrayExtension::S_,
      XrayExtension::H,
      XrayExtension::Vs,
      XrayExtension::Cs,
      XrayExtension::Hs,
      XrayExtension::Ds,
      XrayExtension::Gs,
      XrayExtension::Hlsl,
      XrayExtension::Lua,
    ] {
      assert_eq!(XrayAssetType::of(extension), Some(XrayAssetType::Shader));
    }

    for extension in [XrayExtension::Seq, XrayExtension::Seq_] {
      assert_eq!(XrayAssetType::of(extension), Some(XrayAssetType::Seq));
    }

    for extension in [
      XrayExtension::Log,
      XrayExtension::Bat,
      XrayExtension::Py,
      XrayExtension::Cmd,
    ] {
      assert_eq!(XrayAssetType::of(extension), Some(XrayAssetType::Misc));
    }
  }

  #[test]
  fn ix_rays_renderer_spellings_are_shader_sources_and_not_game_scripts() {
    assert_eq!(XrayAssetType::of(XrayExtension::Lua), Some(XrayAssetType::Shader));
    assert_eq!(XrayAssetType::of(XrayExtension::Hlsl), Some(XrayAssetType::Shader));
    assert_eq!(XrayAssetType::of(XrayExtension::Script), Some(XrayAssetType::Script));

    // The leading-dot renderer script, which IX-Ray ships as `.lua` where the older trees ship `.s`.
    assert_eq!(
      XrayAssetType::from_logical_path("shaders\\r1\\.lua"),
      Some(XrayAssetType::Shader)
    );
    assert_eq!(
      XrayAssetType::from_logical_path("shaders\\d3d11\\accum_base.ps.hlsl"),
      Some(XrayAssetType::Shader)
    );
    // A script the engine's own loader registers keeps its own kind, extension and home.
    assert_eq!(
      XrayAssetType::from_logical_path("scripts\\xr_logic.script"),
      Some(XrayAssetType::Script)
    );
    assert_eq!(
      rules(XrayAssetType::Script).to_logical_path("xr_logic"),
      "xr_logic.script"
    );
  }

  #[test]
  fn names_no_kind_for_a_spelling_no_mounted_world_resolves_by_kind() {
    // `ps_static` is an asset and `png` is not, though both are files a gamedata tree really holds.
    assert_eq!(
      XrayAssetType::of(XrayExtension::PsStatic),
      Some(XrayAssetType::PsStatic)
    );
    assert_eq!(XrayAssetType::of(XrayExtension::Png), None);
    assert_eq!(XrayAssetType::of(XrayExtension::Tga), None);
    assert_eq!(XrayAssetType::of(XrayExtension::Xml), None);
  }

  #[test]
  fn reads_a_kind_off_an_extension_whatever_case_a_name_was_authored_in() {
    // A name table records a name as it was written, and an archive built on Windows holds plenty of upper case.
    assert_eq!(
      XrayAssetType::from_logical_path("TEXTURES\\WPN\\AK74.DDS"),
      Some(XrayAssetType::Dds)
    );
    assert_eq!(
      XrayAssetType::from_logical_path("Shaders\\R1\\.S"),
      Some(XrayAssetType::Shader)
    );
  }
}
