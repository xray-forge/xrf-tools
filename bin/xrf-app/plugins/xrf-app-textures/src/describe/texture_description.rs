use std::path::{Path, PathBuf};

use serde::Serialize;
use xrf_app_core::TauriResult;
use xrf_app_core::assets::{AssetMountState, AssetSource, AssetTextureDescriptor};
use xrf_chunk::XRayByteOrder;
use xrf_material::{XrayMaterialBumpInput, XrayMaterialDescriptor, XrayMaterialResolver, XrayTextureScope};
use xrf_thm::ThmFile;
use xrf_vfs::{XrayAsset, XrayAssetContainer, XrayAssetType, XrayLogicalPath, XrayRoots, XraySearch};

use crate::browse::TextureBumpPair;
use crate::describe::{TextureFiles, TextureSource};
use crate::edit::{TextureDescriptorForm, TextureEditTargets};

/// Everything the inspection panels say about one texture, resolved in one call.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextureDescription {
  pub source: TextureSource,
  /// What to call this texture on screen: its engine reference, or a standalone file's own stem.
  pub reference: String,
  /// The roots the description was resolved in, so a later read searches what this searched.
  pub roots: XrayRoots,
  /// The `.dds` the reference resolves to, or `None` for a descriptor with no texture.
  pub texture: Option<XrayAsset>,
  /// Where the texture's pixels are read from: an asset of `roots`, or a standalone file by its path, which nothing
  /// mounts; `None` for a descriptor with no texture.
  pub texture_source: Option<AssetSource>,
  /// What the base texture file is, when it is located and its bytes can be reached.
  pub base: Option<AssetTextureDescriptor>,
  /// What the renderer would build for this texture, or `None` for a file outside every tree.
  pub material: Option<XrayMaterialDescriptor>,
  /// What the bound bump file is, when the material binds one and its bytes can be reached.
  pub bump: Option<AssetTextureDescriptor>,
  /// What the bound bump companion file is, on the same terms.
  pub companion: Option<AssetTextureDescriptor>,
  /// Whether a `.thm` was located and parsed, so the editor edits one rather than authoring it.
  pub has_descriptor: bool,
  /// The descriptor's editable fields, or the SDK's defaults a descriptor authored for this texture starts from.
  pub form: TextureDescriptorForm,
  /// Where an edit of this texture would write, absent for a texture served out of an archive.
  pub targets: Option<TextureEditTargets>,
  /// The pair the bump generator writes for this texture, as the references a descriptor names.
  pub generated_bump: TextureBumpPair,
}

impl TextureDescription {
  /// Describes a texture through `roots` centred on the file it names, if it names one, as every command describes
  /// one.
  ///
  /// # Errors
  ///
  /// Returns an error when the roots cannot be mounted, a reference is refused by the search, or a standalone source
  /// names no file on disk.
  pub fn describe_in(assets: &AssetMountState, source: TextureSource, roots: XrayRoots) -> TauriResult<Self> {
    let roots: XrayRoots = roots.centred_on(source.physical_path());

    Self::describe(&assets.search(&roots)?, source, roots)
  }

  /// Describes a texture, through the roots where it has an engine reference and from its own path where it has none.
  ///
  /// # Errors
  ///
  /// Returns an error when a reference is refused by the search, or when a standalone source names no file on disk.
  pub fn describe(search: &XraySearch, source: TextureSource, roots: XrayRoots) -> TauriResult<Self> {
    match source.to_reference() {
      Some(reference) => Self::describe_in_roots(search, source, roots, reference),
      None => Self::describe_standalone(source, roots),
    }
  }

  /// Describes a texture the engine has a name for, through the roots that name resolves in.
  fn describe_in_roots(
    search: &XraySearch,
    source: TextureSource,
    roots: XrayRoots,
    reference: String,
  ) -> TauriResult<Self> {
    let texture: Option<XrayAsset> = search
      .resolve(XrayAssetType::Dds, &reference)
      .map_err(|error| format!("Rejected texture '{reference}': {error}"))?
      .get_asset()
      .cloned();
    let material: XrayMaterialDescriptor =
      XrayMaterialResolver::describe_texture(search, &XrayTextureScope::shared(), &reference);

    // Whichever file the engine ended up binding for a half, declared or substituted, is the one worth describing.
    let describe_bound = |input: &XrayMaterialBumpInput| -> Option<AssetTextureDescriptor> {
      AssetTextureDescriptor::describe(search, input.resolution.get_asset()?)
    };

    let descriptor: Option<ThmFile> = material
      .descriptor
      .as_ref()
      .and_then(|asset| Self::read_descriptor(search, asset));

    Ok(Self {
      has_descriptor: descriptor.is_some(),
      form: TextureDescriptorForm::of(descriptor.as_ref()),
      targets: match &texture {
        Some(asset) => TextureEditTargets::of(asset, material.descriptor.as_ref())?,
        None => None,
      },
      base: texture
        .as_ref()
        .and_then(|asset| AssetTextureDescriptor::describe(search, asset)),
      texture_source: texture.as_ref().map(|asset| AssetSource::Asset {
        logical_path: asset.get_logical_path().as_str().to_owned(),
      }),
      bump: material.bump.as_ref().and_then(|bump| describe_bound(&bump.bump)),
      companion: material.bump.as_ref().and_then(|bump| describe_bound(&bump.companion)),
      material: Some(material),
      generated_bump: TextureBumpPair::generated_for(&reference),
      source,
      reference,
      roots,
      texture,
    })
  }

  /// Describes a file that sits under no X-Ray root, from its own path: read straight off disk, never mounted, since
  /// mounting its folder would index everything below it for the life of the process.
  ///
  /// # Errors
  ///
  /// Returns an error when the source names no file on disk, which for a standalone description is the whole address.
  fn describe_standalone(source: TextureSource, roots: XrayRoots) -> TauriResult<Self> {
    let path: PathBuf = source
      .physical_path()
      .ok_or_else(|| String::from("A texture outside every root has to be named by a file path"))?
      .to_path_buf();

    let TextureFiles {
      texture: texture_path,
      descriptor: descriptor_path,
    } = TextureFiles::of(&path);
    let texture: Option<XrayAsset> = texture_path
      .is_file()
      .then(|| Self::to_loose_asset(&texture_path))
      .flatten();
    let descriptor: Option<ThmFile> = Self::read_descriptor_from_path(&descriptor_path);

    Ok(Self {
      has_descriptor: descriptor.is_some(),
      form: TextureDescriptorForm::of(descriptor.as_ref()),
      targets: Some(TextureEditTargets::of_paths(&descriptor_path, &texture_path)?),
      base: texture
        .as_ref()
        .and_then(|_| AssetTextureDescriptor::describe_path(&texture_path)),
      texture_source: texture
        .as_ref()
        .and_then(|_| texture_path.to_str())
        .map(|path| AssetSource::File { path: path.to_owned() }),
      generated_bump: TextureBumpPair::generated_for(&source.to_label()),
      reference: source.to_label(),
      material: None,
      bump: None,
      companion: None,
      source,
      roots,
      texture,
    })
  }

  /// A loose file as the VFS would report it, rooted at its own directory.
  fn to_loose_asset(path: &Path) -> Option<XrayAsset> {
    let name: &str = path.file_name()?.to_str()?;

    Some(XrayAsset::new(
      XrayLogicalPath::new(name).ok()?,
      XrayAssetContainer::Directory {
        root: path.parent()?.to_path_buf(),
        relative_path: PathBuf::from(name),
      },
    ))
  }

  /// Reads a located `.thm`, or nothing when its bytes cannot be reached or are not a descriptor.
  fn read_descriptor(search: &XraySearch, asset: &XrayAsset) -> Option<ThmFile> {
    ThmFile::read_from_bytes::<XRayByteOrder>(search.read_asset(asset).ok()?).ok()
  }

  /// Reads a `.thm` straight off disk, for a file no mount holds.
  fn read_descriptor_from_path(path: &Path) -> Option<ThmFile> {
    path
      .is_file()
      .then(|| ThmFile::read_from_path::<XRayByteOrder, _>(&path).ok())
      .flatten()
  }
}
