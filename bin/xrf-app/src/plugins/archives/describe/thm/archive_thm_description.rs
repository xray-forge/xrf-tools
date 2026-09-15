use serde::Serialize;
use xrf_db::{
  ThmBumpChunk, ThmDetailChunk, ThmFile, ThmMaterialChunk, ThmTextureFlag, ThmTextureParamChunk, ThmTextureType,
  XRayByteOrder,
};
use xrf_error::XrfResult;
use xrf_vfs::{XrayAssetType, XrayLogicalPath};

use crate::core::assets::AssetTextureShape;
use crate::plugins::archives::describe::archive_describe_source::ArchiveDescribeSource;
use crate::plugins::archives::describe::archive_reference::ArchiveReference;

/// Everything the viewer says about one texture descriptor.
///
/// Field order is reading order, and reading order is the engine's rather than the file's: what the descriptor
/// describes, whether the engine reads it at all, what it names, how it shades, and only then the build recipe the
/// converter already consumed. The file's own chunk order puts the recipe third, which is the order to write it back
/// in and not the order to read it in.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveThmDescription {
  pub texture: ArchiveThmTexture,
  pub texture_type: ArchiveThmTextureType,
  pub bump: Option<ArchiveThmBump>,
  pub detail: Option<ArchiveThmDetail>,
  /// The normal map replacing the one the generator would derive, when the file names one.
  pub external_normal_map: Option<ArchiveReference>,
  pub material: Option<ArchiveThmMaterial>,
  pub parameters: Option<ArchiveThmParameters>,
  /// Mip level the converter starts fading from, `fade_delay` (`ETextureParams.h:88`).
  pub fade_delay: Option<u8>,
  pub file: ArchiveThmFile,
}

/// The texture a descriptor sits beside, and what that file actually is.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveThmTexture {
  /// The `.dds` this descriptor describes, which is the file beside it rather than a name it carries.
  pub reference: ArchiveReference,
  /// What that file's header declares, when it was found and its header parsed.
  pub shape: Option<AssetTextureShape>,
  /// The size the descriptor claims, carried only when the file beside it measures something else.
  pub declared: Option<ArchiveThmDeclaredSize>,
}

/// A declared size the texture beside the descriptor does not match.
///
/// Reported as two facts rather than as a fault. The descriptor's width and height are authoring data and the file is
/// the authority (`STextureParams` records what was converted, not what came out), so a disagreement is worth seeing
/// and is not by itself wrong.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveThmDeclaredSize {
  pub width: u32,
  pub height: u32,
  /// Whether the declaration is a cube map's source strip: six faces wide, one face tall.
  pub is_cube_strip: bool,
}

/// The kind of texture a descriptor describes, `STextureParams::ETType`.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveThmTextureType {
  /// Engine token for the type, or the raw number for one the SDK does not name.
  pub label: String,
  pub value: u32,
  /// Whether `CTextureDescrMngr::LoadTHM` reads the bump, detail and material of a descriptor of this type at all.
  ///
  /// False for 743 of vanilla's 2,736 descriptors - every cube map and every bump map - whose declarations the engine
  /// never looks at however complete they are.
  pub is_read_by_engine: bool,
  /// Whether the file declares a type, or the engine's zeroed default is what applies.
  pub is_declared: bool,
}

/// The bump declaration of a descriptor, `THM_CHUNK_BUMP`.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveThmBump {
  pub mode_label: String,
  pub mode: u32,
  /// Height the generator builds the pair against, read at generation time and never at runtime.
  pub virtual_height: f32,
  /// The bump texture named, absent when the chunk names none.
  pub texture: Option<ArchiveReference>,
  /// Whether the engine would try to resolve the name: a mode that uses one, and a name to use.
  ///
  /// A name that resolves to nothing does not turn bump mapping off. `bump_exist` tests only that the name is
  /// non-empty, so the renderer still takes the `_bump` variant and the loader substitutes `ed\ed_dummy_bump`.
  pub is_used: bool,
}

/// The detail association of a descriptor, `THM_CHUNK_DETAIL_EXT`.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveThmDetail {
  pub scale: f32,
  /// The detail texture named, absent when the chunk names none.
  pub texture: Option<ArchiveReference>,
  /// The flags that switch this association on, by the SDK's own spelling.
  ///
  /// Empty when neither applies, which is a name the engine reads past (`TextureDescrManager.cpp:163`). Repeated here
  /// as well as in the flag word because the association means nothing without them.
  pub enabled_by: Vec<String>,
}

/// The shading declaration of a descriptor, `THM_CHUNK_MATERIAL`.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveThmMaterial {
  /// The two lighting models the surface sits between.
  pub label: String,
  pub value: u32,
  /// Where between them it sits.
  pub weight: f32,
}

/// The conversion parameters a descriptor carries, `THM_CHUNK_TEXTUREPARAM`.
///
/// Authoring data the converter consumed and the runtime does not read, with two exceptions that live in
/// [`ArchiveThmDetail::enabled_by`].
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveThmParameters {
  pub format_label: String,
  pub format: u32,
  pub mip_filter_label: String,
  pub mip_filter: u32,
  pub border_color: u32,
  pub fade_color: u32,
  pub fade_amount: u32,
  pub width: u32,
  pub height: u32,
  /// Every bit the SDK names, in bit order, set or not.
  ///
  /// All twelve rather than the set ones: a recipe is read to see what the converter was told to do, and "dither is
  /// off" answers that as well as "dither is on".
  pub flags: Vec<ArchiveThmFlag>,
  /// Bits the word carries that the SDK has no name for.
  ///
  /// Two vanilla descriptors carry one, so dropping the residue would silently lose what somebody's tool set.
  pub unnamed_flags: u32,
}

/// One bit of the texture param flag word.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveThmFlag {
  /// SDK identifier, which is the name an author of a `.thm` would recognise.
  pub label: String,
  pub is_set: bool,
}

/// What the file is, apart from what it declares.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveThmFile {
  pub version: Option<u16>,
  /// Whether the version is the one `ETextureThumbnail::Load` accepts.
  pub is_supported_version: bool,
  /// The kind of asset the thumbnail describes; `1` is a texture and the only one these tools read.
  pub thumbnail_type: Option<u32>,
  pub thumbnail: Option<ArchiveThmThumbnail>,
  /// Chunk ids the reader could not fold into a field, in the order it read them.
  pub extra_chunks: Vec<u32>,
}

/// The preview picture a descriptor carries, `THM_CHUNK_DATA`.
///
/// Reported by size and never decoded. The trunk SDK stopped writing the chunk - its `w_chunk` call is commented out
/// in `ETextureThumbnail::Save` - and 11 of the 19,849 descriptors across the workspace trees still carry one.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveThmThumbnail {
  /// Whether the payload is the engine's own compressed stream, which is the only form seen in the wild.
  pub is_compressed: bool,
  pub size: u64,
}

impl ArchiveThmDescription {
  /// Reads the descriptor an entry holds and resolves what it names against the subject being browsed.
  ///
  /// # Errors
  ///
  /// Returns an error when the entry's bytes cannot be read, or when they are not a descriptor this reader can walk.
  /// A chunk it does not recognise is not a failure: [`ThmFile`] keeps it, and it is reported as an extra chunk.
  pub fn read(source: &ArchiveDescribeSource, name: &str) -> XrfResult<Self> {
    let file: ThmFile = ThmFile::read_from_bytes::<XRayByteOrder>(source.read_bytes(name)?)?;
    let parameters: Option<&ThmTextureParamChunk> = file.texture_param.as_ref();

    Ok(Self {
      texture: describe_texture(source, name, file.texture_type(), parameters),
      texture_type: describe_texture_type(&file),
      bump: file.bump.as_ref().map(|bump| describe_bump(source, bump)),
      detail: file
        .detail
        .as_ref()
        .map(|detail| describe_detail(source, detail, parameters)),
      external_normal_map: file
        .ext_normal_map_name
        .as_deref()
        .filter(|name| !name.is_empty())
        .map(|name| ArchiveReference::resolve(source, XrayAssetType::Dds, name)),
      material: file.material.as_ref().map(describe_material),
      parameters: parameters.map(describe_parameters),
      fade_delay: file.fade_delay,
      file: describe_file(&file),
    })
  }
}

/// The texture beside the descriptor, with the declared size when the file disagrees with it.
fn describe_texture(
  source: &ArchiveDescribeSource,
  name: &str,
  texture_type: ThmTextureType,
  parameters: Option<&ThmTextureParamChunk>,
) -> ArchiveThmTexture {
  let reference: ArchiveReference = match to_sibling_texture_path(name) {
    Some(path) => ArchiveReference::of_path(source, &path),
    None => ArchiveReference::unresolvable(name),
  };

  let shape: Option<AssetTextureShape> = reference
    .entry
    .as_deref()
    .and_then(|entry| source.describe_texture(entry));

  ArchiveThmTexture {
    declared: describe_declared_size(texture_type, parameters, shape.as_ref()),
    reference,
    shape,
  }
}

/// The declared size, when there is a file to compare it against and the two differ.
fn describe_declared_size(
  texture_type: ThmTextureType,
  parameters: Option<&ThmTextureParamChunk>,
  shape: Option<&AssetTextureShape>,
) -> Option<ArchiveThmDeclaredSize> {
  let parameters: &ThmTextureParamChunk = parameters?;
  let shape: &AssetTextureShape = shape?;

  if parameters.width == shape.width && parameters.height == shape.height {
    return None;
  }

  Some(ArchiveThmDeclaredSize {
    width: parameters.width,
    height: parameters.height,
    is_cube_strip: texture_type == ThmTextureType::CubeMap
      && parameters.height == shape.height
      && parameters.width == shape.width.saturating_mul(6),
  })
}

fn describe_texture_type(file: &ThmFile) -> ArchiveThmTextureType {
  let texture_type: ThmTextureType = file.texture_type();

  ArchiveThmTextureType {
    label: texture_type.label(),
    value: texture_type.into(),
    is_read_by_engine: texture_type.is_described_by_engine(),
    is_declared: file.texture_type.is_some(),
  }
}

fn describe_bump(source: &ArchiveDescribeSource, bump: &ThmBumpChunk) -> ArchiveThmBump {
  ArchiveThmBump {
    mode_label: bump.mode.label(),
    mode: bump.mode.into(),
    virtual_height: bump.virtual_height,
    texture: (!bump.name.is_empty()).then(|| ArchiveReference::resolve(source, XrayAssetType::Dds, &bump.name)),
    is_used: bump.is_used(),
  }
}

fn describe_detail(
  source: &ArchiveDescribeSource,
  detail: &ThmDetailChunk,
  parameters: Option<&ThmTextureParamChunk>,
) -> ArchiveThmDetail {
  ArchiveThmDetail {
    scale: detail.scale,
    texture: (!detail.name.is_empty()).then(|| ArchiveReference::resolve(source, XrayAssetType::Dds, &detail.name)),
    enabled_by: [ThmTextureFlag::DiffuseDetail, ThmTextureFlag::BumpDetail]
      .into_iter()
      .filter(|flag| parameters.is_some_and(|parameters| parameters.flags.has(*flag)))
      .map(|flag| flag.label().to_owned())
      .collect(),
  }
}

fn describe_material(material: &ThmMaterialChunk) -> ArchiveThmMaterial {
  ArchiveThmMaterial {
    label: material.material.label(),
    value: material.material.into(),
    weight: material.weight,
  }
}

fn describe_parameters(parameters: &ThmTextureParamChunk) -> ArchiveThmParameters {
  ArchiveThmParameters {
    format_label: parameters.format.label(),
    format: parameters.format.into(),
    mip_filter_label: parameters.mip_filter.label(),
    mip_filter: parameters.mip_filter.into(),
    border_color: parameters.border_color,
    fade_color: parameters.fade_color,
    fade_amount: parameters.fade_amount,
    width: parameters.width,
    height: parameters.height,
    flags: ThmTextureFlag::NAMED
      .into_iter()
      .map(|flag| ArchiveThmFlag {
        label: flag.label().to_owned(),
        is_set: parameters.flags.has(flag),
      })
      .collect(),
    unnamed_flags: parameters.flags.unnamed(),
  }
}

fn describe_file(file: &ThmFile) -> ArchiveThmFile {
  ArchiveThmFile {
    version: file.version,
    is_supported_version: file.is_supported_version(),
    thumbnail_type: file.thumbnail_type,
    thumbnail: file.thumbnail.as_ref().map(|thumbnail| ArchiveThmThumbnail {
      is_compressed: thumbnail.is_compressed,
      size: thumbnail.data.len() as u64,
    }),
    extra_chunks: file.extra.iter().map(|chunk| chunk.id).collect(),
  }
}

/// The `.dds` beside a descriptor, addressed by where it sits rather than by a name the descriptor carries.
///
/// A descriptor names its bump and its detail; it does not name its own texture. The engine pairs the two by path
/// (`CTextureDescrMngr::LoadTHM` walks descriptors and keys them by the reference the `.thm` is filed under), so the
/// sibling is the same path with the loaded extension - which also answers for a descriptor outside `textures\`,
/// where no engine reference exists to resolve.
fn to_sibling_texture_path(name: &str) -> Option<String> {
  let path: XrayLogicalPath = XrayLogicalPath::new(name).ok()?;

  path.as_str().strip_suffix(".thm").map(|stem| format!("{stem}.dds"))
}

#[cfg(test)]
mod tests {
  use xrf_db::{ThmTextureParamChunk, ThmTextureType};

  use super::{describe_declared_size, to_sibling_texture_path};
  use crate::core::assets::AssetTextureShape;

  fn shape(width: u32, height: u32) -> AssetTextureShape {
    AssetTextureShape {
      width,
      height,
      mipmap_levels: 1,
      format: String::from("DXT1"),
    }
  }

  fn parameters(width: u32, height: u32) -> ThmTextureParamChunk {
    ThmTextureParamChunk {
      width,
      height,
      ..ThmTextureParamChunk::default()
    }
  }

  #[test]
  fn a_descriptor_agreeing_with_its_texture_declares_nothing_extra() {
    assert_eq!(
      describe_declared_size(
        ThmTextureType::Image,
        Some(&parameters(512, 512)),
        Some(&shape(512, 512))
      ),
      None
    );
  }

  #[test]
  fn a_cube_maps_source_strip_is_recognised_rather_than_reported_as_a_disagreement() {
    // `sky\sky_12_cube.thm` declares the six-face strip it was converted from; the file is the packed cube. 53 of
    // vanilla's 54 disagreements are this, so failing to say so would make the field noise.
    let declared = describe_declared_size(
      ThmTextureType::CubeMap,
      Some(&parameters(3072, 512)),
      Some(&shape(512, 512)),
    )
    .expect("the sizes differ");

    assert_eq!((declared.width, declared.height), (3072, 512));
    assert!(declared.is_cube_strip);
  }

  #[test]
  fn a_plain_disagreement_is_carried_without_an_explanation() {
    // `ui\ui_grid.thm`, the one vanilla descriptor that genuinely disagrees with the texture beside it.
    let declared = describe_declared_size(ThmTextureType::Image, Some(&parameters(128, 64)), Some(&shape(256, 64)))
      .expect("the sizes differ");

    assert!(!declared.is_cube_strip);
  }

  #[test]
  fn a_six_wide_declaration_is_only_a_strip_for_a_cube_map() {
    let declared = describe_declared_size(
      ThmTextureType::Image,
      Some(&parameters(3072, 512)),
      Some(&shape(512, 512)),
    )
    .expect("the sizes differ");

    assert!(!declared.is_cube_strip, "the type is what makes the strip meaningful");
  }

  #[test]
  fn nothing_is_declared_without_a_texture_to_compare_against() {
    assert_eq!(
      describe_declared_size(ThmTextureType::Image, Some(&parameters(512, 512)), None),
      None
    );
  }

  #[test]
  fn a_descriptor_names_the_texture_it_sits_beside() {
    assert_eq!(
      to_sibling_texture_path("textures\\act\\act_arm_1.thm").as_deref(),
      Some("textures\\act\\act_arm_1.dds")
    );
    assert_eq!(
      to_sibling_texture_path("Textures\\Act\\ACT_ARM_1.THM").as_deref(),
      Some("textures\\act\\act_arm_1.dds")
    );
  }
}
