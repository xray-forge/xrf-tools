use std::sync::Arc;

use xrf_chunk::ChunkReader;
use xrf_db::{ThmBumpChunk, ThmFile, ThmTextureTypeChunk, XRayByteOrder};
use xrf_error::XrfResult;
use xrf_vfs::{XrayAsset, XrayAssetType, XrayProbe, XrayResolution};

use crate::data::xray_bump_fallback::XrayBumpFallback;
use crate::data::xray_bump_mode::XrayBumpMode;
use crate::data::xray_bump_outcome::XrayBumpOutcome;
use crate::data::xray_material_bump::XrayMaterialBump;
use crate::data::xray_material_bump_input::XrayMaterialBumpInput;
use crate::data::xray_material_declaration::XrayMaterialDeclaration;
use crate::data::xray_material_descriptor::XrayMaterialDescriptor;
use crate::data::xray_material_detail::XrayMaterialDetail;

/// Reads a texture's descriptor and resolves what the renderer would bind from it.
///
/// Two doors on one classification. [`Self::describe_texture`] starts from a texture reference and finds its `.thm`,
/// which is what a viewer holding a mesh's texture names wants and what `CTexture::Preload` does.
/// [`Self::describe_descriptor`] starts from a `.thm` already in hand, which is what a sweep enumerating descriptors
/// wants: the engine's `LoadTHM` walks descriptors, not textures, so a descriptor with no texture beside it is still
/// checked.
///
/// Resolves through a borrowed probe and never mounts: which roots exist, in what order, is the caller's policy.
pub struct XrayMaterialResolver;

impl XrayMaterialResolver {
  /// Describes the material of a texture named the way a mesh or a config names it.
  pub fn describe_texture(probe: &XrayProbe, reference: &str) -> XrayMaterialDescriptor {
    match probe.resolve(XrayAssetType::Thm, reference) {
      Ok(XrayResolution::Resolved { assets, .. }) => match assets.first() {
        Some(asset) => Self::describe_descriptor(probe, asset),
        None => XrayMaterialDescriptor::undeclared(),
      },
      Ok(_) | Err(_) => XrayMaterialDescriptor::undeclared(),
    }
  }

  /// Describes the material a located descriptor declares.
  ///
  /// Reads in the engine's order: the file has to parse, its type has to be one `LoadTHM` reads, and only then do the
  /// bump and detail chunks mean anything.
  pub fn describe_descriptor(probe: &XrayProbe, descriptor: &XrayAsset) -> XrayMaterialDescriptor {
    let file: Arc<ThmFile> = match Self::read(probe, descriptor) {
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
      let texture_type: u32 = file.texture_type();

      return XrayMaterialDescriptor::flat(
        Some(descriptor.clone()),
        XrayMaterialDeclaration::TypeDisqualified {
          texture_type,
          label: ThmTextureTypeChunk::label(texture_type),
          declared_bump: file.used_bump_name().map(str::to_owned),
        },
      );
    }

    let (declaration, bump) = Self::classify_bump(probe, file.bump.as_ref());
    let outcome: XrayBumpOutcome = bump.as_ref().map_or(XrayBumpOutcome::Flat, |bump| {
      XrayBumpOutcome::of_pair(&bump.bump.resolution, &bump.companion.resolution)
    });

    XrayMaterialDescriptor {
      descriptor: Some(descriptor.clone()),
      declaration,
      bump,
      outcome,
      detail: Self::describe_detail(&file),
    }
  }

  fn read(probe: &XrayProbe, descriptor: &XrayAsset) -> XrfResult<Arc<ThmFile>> {
    probe.read_asset_parsed(descriptor, |bytes| {
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
    probe: &XrayProbe,
    chunk: Option<&ThmBumpChunk>,
  ) -> (XrayMaterialDeclaration, Option<XrayMaterialBump>) {
    let Some(chunk) = chunk else {
      return (XrayMaterialDeclaration::NoBumpChunk, None);
    };

    let Some(mode) = XrayBumpMode::of(chunk.mode) else {
      return (XrayMaterialDeclaration::Disabled { mode: chunk.mode }, None);
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
        bump: Self::resolve_input(probe, chunk.name.clone(), false),
        companion: Self::resolve_input(probe, format!("{}#", chunk.name), true),
      }),
    )
  }

  /// Resolves one bound input, substituting what `texture_load` would for its name.
  ///
  /// A name no logical path can be made of is a rejected outcome for that input alone.
  fn resolve_input(probe: &XrayProbe, reference: String, is_companion: bool) -> XrayMaterialBumpInput {
    let fallback: XrayBumpFallback = XrayBumpFallback::for_input(&reference, is_companion);
    let resolution: XrayResolution = probe
      .resolve_with_fallback(XrayAssetType::Dds, &reference, fallback.reference())
      .unwrap_or_else(|error| XrayResolution::Rejected {
        reason: error.to_string(),
      });

    XrayMaterialBumpInput { reference, resolution }
  }
}
