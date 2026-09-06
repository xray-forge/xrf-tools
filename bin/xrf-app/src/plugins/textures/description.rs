use serde::Serialize;
use xrf_db::{ThmFile, XRayByteOrder};
use xrf_material::{XrayMaterialBumpInput, XrayMaterialDescriptor, XrayMaterialResolver};
use xrf_vfs::{XrayAsset, XrayAssetType, XrayProbe, XrayRoots};

use crate::core::assets::AssetTextureDescriptor;
use crate::core::types::TauriResult;
use crate::plugins::textures::descriptor_form::TextureDescriptorForm;
use crate::plugins::textures::edit_targets::TextureEditTargets;
use crate::plugins::textures::source::TextureSource;

/// Everything the inspection panel says about one texture, resolved in one call.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureDescription {
  pub source: TextureSource,
  /// The engine reference the source came to.
  pub reference: String,
  /// The roots the description was resolved in, so a later read searches what this searched.
  pub roots: XrayRoots,
  /// The `.dds` the reference resolves to, or `None` for a descriptor with no texture.
  pub texture: Option<XrayAsset>,
  /// What the base texture file is, when it is located and its bytes can be reached.
  pub base: Option<AssetTextureDescriptor>,
  pub material: XrayMaterialDescriptor,
  /// What the bound bump file is, when the material binds one and its bytes can be reached.
  pub bump: Option<AssetTextureDescriptor>,
  /// What the bound bump companion file is, on the same terms.
  pub companion: Option<AssetTextureDescriptor>,
  /// The descriptor's editable fields, when a `.thm` was located and parsed.
  ///
  /// Separate from [`Self::material`], which is what the renderer makes of the descriptor. This is the descriptor
  /// itself, and the editor binds to it. A `.thm` that will not parse reports `None` here and its refusal there.
  pub form: Option<TextureDescriptorForm>,
  /// Where an edit of this texture would write, absent for a texture served out of an archive.
  pub targets: Option<TextureEditTargets>,
}

impl TextureDescription {
  /// Describes a texture through one probe, so the texture, its descriptor and the pair are all looked for in the same
  /// roots.
  ///
  /// # Errors
  ///
  /// Returns an error when the source names no engine reference, or when the reference is not an X-Ray path.
  pub fn describe(probe: &XrayProbe, source: TextureSource, roots: XrayRoots) -> TauriResult<Self> {
    let reference: String = source.to_reference()?;
    let texture: Option<XrayAsset> = probe
      .resolve(XrayAssetType::Dds, &reference)
      .map_err(|error| format!("Rejected texture '{reference}': {error}"))?
      .get_asset()
      .cloned();
    let material: XrayMaterialDescriptor = XrayMaterialResolver::describe_texture(probe, &reference);

    // Whichever file the engine ended up binding for a half, declared or substituted, is the one worth describing.
    let describe_bound = |input: &XrayMaterialBumpInput| -> Option<AssetTextureDescriptor> {
      AssetTextureDescriptor::describe(probe, input.resolution.get_asset()?)
    };

    Ok(Self {
      form: material
        .descriptor
        .as_ref()
        .and_then(|asset| read_descriptor(probe, asset))
        .as_ref()
        .map(TextureDescriptorForm::read),
      targets: match &texture {
        Some(asset) => TextureEditTargets::of(asset, material.descriptor.as_ref())?,
        None => None,
      },
      base: texture
        .as_ref()
        .and_then(|asset| AssetTextureDescriptor::describe(probe, asset)),
      bump: material.bump.as_ref().and_then(|bump| describe_bound(&bump.bump)),
      companion: material.bump.as_ref().and_then(|bump| describe_bound(&bump.companion)),
      source,
      reference,
      roots,
      texture,
      material,
    })
  }
}

/// Read a located `.thm`, or nothing when its bytes cannot be reached or are not a descriptor.
fn read_descriptor(probe: &XrayProbe, asset: &XrayAsset) -> Option<ThmFile> {
  ThmFile::read_from_bytes::<XRayByteOrder>(probe.read_asset_bytes(asset).ok()?).ok()
}
