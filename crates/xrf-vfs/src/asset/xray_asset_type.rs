use serde::{Deserialize, Serialize};
use xrf_extension::{XrayExtension, XrayExtensionOf};

use crate::XrayAssetRules;

/// Asset category inferred from an X-Ray logical path's extension or recognized suffix.
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

impl XrayAssetType {
  /// The X-Ray shader library's fixed gamedata-relative logical path.
  pub const SHADER_LIBRARY_PATH: &'static str = super::shader::SHADER_LIBRARY_LOGICAL_PATH;

  /// Converts a sound reference into the name the engine registers it under.
  pub fn sound_reference_name(reference: &str) -> String {
    super::sound::sound_reference_name(reference)
  }

  /// Where this kind lives, for the kinds with one canonical home.
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
      | XrayExtension::Txt
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

#[cfg(test)]
mod tests {
  use xrf_extension::XrayExtension;

  use super::{XrayAssetRules, XrayAssetType};

  fn rules(asset_type: XrayAssetType) -> XrayAssetRules {
    asset_type.get_rules().expect("kind has a canonical home")
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
    // reader outside a VFS snapshot is free to treat them differently.
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
