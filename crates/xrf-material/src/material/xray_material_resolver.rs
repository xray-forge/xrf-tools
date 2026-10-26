use std::sync::Arc;

use xrf_chunk::{ChunkReader, XRayByteOrder};
use xrf_error::XrfResult;
use xrf_thm::{ThmBumpChunk, ThmFile, ThmMaterialChunk, ThmTextureType};
use xrf_vfs::{XrayAsset, XrayAssetType, XrayResolution, XraySearch};

use crate::material::{
  XrayBumpFallback, XrayBumpMode, XrayBumpNaming, XrayBumpOutcome, XrayMaterialBump, XrayMaterialBumpInput,
  XrayMaterialDeclaration, XrayMaterialDescriptor, XrayMaterialDetail,
};
use crate::texture::XrayTextureScope;

/// Reads a texture's descriptor and resolves what the renderer would bind from it.
pub struct XrayMaterialResolver;

impl XrayMaterialResolver {
  /// Where `CTextureDescrMngr::LoadLTX` looks for the texture specification file: `textures.ltx` under
  /// `$game_textures$` (`TextureDescrManager.cpp:37`).
  pub const TEXTURES_LTX_LOGICAL_PATH: &'static str = "textures\\textures.ltx";

  /// The `textures.ltx` the engine would load beside the descriptors, when the roots hold one.
  pub fn find_textures_ltx(search: &XraySearch) -> Option<XrayAsset> {
    search.find(Self::TEXTURES_LTX_LOGICAL_PATH).ok().flatten()
  }

  /// Describes the material of a texture named the way a mesh or a config names it, as the renderer holds it within
  /// `scope`: `LoadTHM` reads the shared tree's descriptors and then the level's over them (`TextureDescrManager.cpp`).
  pub fn describe_texture(search: &XraySearch, scope: &XrayTextureScope, reference: &str) -> XrayMaterialDescriptor {
    let shared: Option<XrayMaterialDescriptor> = search
      .resolve(XrayAssetType::Thm, reference)
      .ok()
      .and_then(|resolution| resolution.get_asset().cloned())
      .map(|asset| Self::describe_within(search, scope, &asset));
    let level: Option<XrayMaterialDescriptor> = scope
      .find_beside_level(search, XrayAssetType::Thm, reference)
      .and_then(|resolution| resolution.get_asset().cloned())
      .map(|asset| Self::describe_within(search, scope, &asset));

    match (level, shared) {
      (Some(level), Some(shared)) => Self::layer(level, shared),
      (Some(described), None) | (None, Some(described)) => described,
      (None, None) => XrayMaterialDescriptor::undeclared(),
    }
  }

  /// Describes the material a located descriptor declares, outside any level.
  pub fn describe_descriptor(search: &XraySearch, descriptor: &XrayAsset) -> XrayMaterialDescriptor {
    Self::describe_within(search, &XrayTextureScope::shared(), descriptor)
  }

  /// Describes the material a located descriptor declares, outside any level, from the read a caller already made.
  pub fn describe_read_descriptor(
    search: &XraySearch,
    descriptor: &XrayAsset,
    file: &XrfResult<Arc<ThmFile>>,
  ) -> XrayMaterialDescriptor {
    Self::describe_read(search, &XrayTextureScope::shared(), descriptor, file)
  }

  /// What `LoadTHM` leaves for one name once a level's descriptor is read over the shared one: one the type gate skips
  /// changes nothing, and one it reads replaces the bump and the material whole and the detail only where it names a
  /// live one.
  fn layer(level: XrayMaterialDescriptor, shared: XrayMaterialDescriptor) -> XrayMaterialDescriptor {
    if level.is_engine_skipped() {
      return shared;
    }

    if level.is_unreadable() || level.is_detail_associated() || !shared.is_detail_associated() {
      return level;
    }

    XrayMaterialDescriptor {
      detail: shared.detail,
      ..level
    }
  }

  /// Describes a located descriptor, resolving what it declares within `scope`.
  fn describe_within(search: &XraySearch, scope: &XrayTextureScope, descriptor: &XrayAsset) -> XrayMaterialDescriptor {
    Self::describe_read(search, scope, descriptor, &Self::read_descriptor(search, descriptor))
  }

  /// Describes a located descriptor from its read, resolving what it declares within `scope`.
  fn describe_read(
    search: &XraySearch,
    scope: &XrayTextureScope,
    descriptor: &XrayAsset,
    file: &XrfResult<Arc<ThmFile>>,
  ) -> XrayMaterialDescriptor {
    let file: &ThmFile = match file {
      Ok(file) => file,
      Err(error) => {
        return XrayMaterialDescriptor::flat(
          Some(descriptor.clone()),
          XrayMaterialDeclaration::Unreadable {
            reason: error.to_string(),
          },
        );
      }
    };

    if !file.is_described_by_engine() {
      let texture_type: ThmTextureType = file.texture_type();

      return XrayMaterialDescriptor::flat(
        Some(descriptor.clone()),
        XrayMaterialDeclaration::TypeDisqualified {
          texture_type: texture_type.into(),
          label: texture_type.label(),
          declared_bump: file.used_bump_name().map(str::to_owned),
        },
      );
    }

    let (declaration, bump) = Self::classify_bump(search, scope, file.bump.as_ref());
    let outcome: XrayBumpOutcome = bump.as_ref().map_or(XrayBumpOutcome::Flat, |bump| {
      XrayBumpOutcome::of_pair(&bump.bump.resolution, &bump.companion.resolution)
    });

    XrayMaterialDescriptor {
      descriptor: Some(descriptor.clone()),
      declaration,
      bump,
      outcome,
      detail: Self::describe_detail(file),
      material: Self::describe_material(file.material.unwrap_or_default()),
    }
  }

  /// `float(tp.material) + tp.material_weight`, a chunk the descriptor lacks read as its defaults.
  fn describe_material(chunk: ThmMaterialChunk) -> f32 {
    u32::from(chunk.material) as f32 + chunk.weight
  }

  /// A located descriptor as the engine's reader parses it, through the search's cache.
  pub fn read_descriptor(search: &XraySearch, descriptor: &XrayAsset) -> XrfResult<Arc<ThmFile>> {
    search.read_parsed(descriptor, |bytes| {
      ThmFile::read_from_chunk::<XRayByteOrder, _>(&mut ChunkReader::from_vec(bytes)?)
    })
  }

  /// The detail association the descriptor names, dead or live, for a descriptor the engine reads.
  fn describe_detail(file: &ThmFile) -> Option<XrayMaterialDetail> {
    file
      .detail
      .as_ref()
      .filter(|detail| !detail.name.is_empty())
      .map(|detail| XrayMaterialDetail {
        name: detail.name.clone(),
        scale: detail.scale,
        usage: file.used_detail_usage().map(Into::into),
      })
  }

  /// The bump chunk as the engine reads it, and the pair it binds when there is one to bind.
  fn classify_bump(
    search: &XraySearch,
    scope: &XrayTextureScope,
    chunk: Option<&ThmBumpChunk>,
  ) -> (XrayMaterialDeclaration, Option<XrayMaterialBump>) {
    let Some(chunk) = chunk else {
      return (XrayMaterialDeclaration::NoBumpChunk, None);
    };

    let Some(mode) = XrayBumpMode::of(chunk.mode) else {
      return (
        XrayMaterialDeclaration::Disabled {
          mode: chunk.mode.into(),
        },
        None,
      );
    };

    if chunk.name.is_empty() {
      return (XrayMaterialDeclaration::EmptyName { mode }, None);
    }

    (
      XrayMaterialDeclaration::Declared {
        mode,
        name: chunk.name.clone(),
      },
      Some(XrayMaterialBump {
        mode,
        virtual_height: chunk.virtual_height,
        bump: Self::resolve_input(search, scope, chunk.name.clone(), false),
        companion: Self::resolve_input(search, scope, XrayBumpNaming::companion_of(&chunk.name), true),
      }),
    )
  }

  /// Resolves one bound input, substituting what `texture_load` would for its name.
  fn resolve_input(
    search: &XraySearch,
    scope: &XrayTextureScope,
    reference: String,
    is_companion: bool,
  ) -> XrayMaterialBumpInput {
    let fallback: XrayBumpFallback = XrayBumpFallback::for_input(&reference, is_companion);
    let resolution: XrayResolution = scope
      .resolve_bump_input(search, &reference, fallback.reference())
      .unwrap_or_else(|error| XrayResolution::Rejected {
        reason: error.to_string(),
      });

    XrayMaterialBumpInput { reference, resolution }
  }
}
