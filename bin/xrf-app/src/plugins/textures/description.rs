use std::path::{Path, PathBuf};

use serde::Serialize;
use xrf_db::{ThmFile, XRayByteOrder};
use xrf_material::{XrayMaterialBumpInput, XrayMaterialDescriptor, XrayMaterialResolver};
use xrf_vfs::{
  XrayAsset, XrayAssetContainer, XrayAssetType, XrayLogicalPath, XrayMountMode, XrayProbe, XrayRoot, XrayRoots,
};

use crate::core::assets::AssetTextureDescriptor;
use crate::core::types::TauriResult;
use crate::plugins::textures::descriptor_form::TextureDescriptorForm;
use crate::plugins::textures::edit_targets::TextureEditTargets;
use crate::plugins::textures::source::TextureSource;

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
  /// What the base texture file is, when it is located and its bytes can be reached.
  pub base: Option<AssetTextureDescriptor>,
  /// What the renderer would build for this texture, or `None` for a file outside every tree.
  ///
  /// Absent rather than empty for a standalone file. There is no tree to resolve a bump pair or a detail against, so
  /// answering "declares nothing" would be a claim this description is in no position to make - the descriptor beside
  /// the file may well declare a pair, and what the engine would do with it depends on a game tree nobody has named.
  pub material: Option<XrayMaterialDescriptor>,
  /// What the bound bump file is, when the material binds one and its bytes can be reached.
  pub bump: Option<AssetTextureDescriptor>,
  /// What the bound bump companion file is, on the same terms.
  pub companion: Option<AssetTextureDescriptor>,
  /// The descriptor's editable fields, when a `.thm` was located and parsed.
  pub form: Option<TextureDescriptorForm>,
  /// Where an edit of this texture would write, absent for a texture served out of an archive.
  pub targets: Option<TextureEditTargets>,
}

impl TextureDescription {
  /// Describes a texture, through the roots where it has an engine reference and from its own path where it has none.
  ///
  /// The two shapes answer different questions, which is why they are two. A texture inside a tree is described as the
  /// engine would find it: one probe locates the file, its descriptor and both halves of any declared pair, so all
  /// four are looked for in the same roots and a second probe cannot mount something between the calls. A file outside
  /// every tree has no engine reference at all, so there is nothing to resolve and nothing to resolve it against; it
  /// is described from the bytes on disk.
  ///
  /// # Errors
  ///
  /// Returns an error when a reference is refused by the probe, or when a standalone source names no file on disk.
  pub fn describe(probe: &XrayProbe, source: TextureSource, roots: XrayRoots) -> TauriResult<Self> {
    match source.to_reference() {
      Some(reference) => Self::describe_in_roots(probe, source, roots, reference),
      None => Self::describe_standalone(source, roots),
    }
  }

  /// Describes a texture the engine has a name for, through the roots that name resolves in.
  fn describe_in_roots(
    probe: &XrayProbe,
    source: TextureSource,
    roots: XrayRoots,
    reference: String,
  ) -> TauriResult<Self> {
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
      material: Some(material),
      source,
      reference,
      roots,
      texture,
    })
  }

  /// Describes a file that sits under no X-Ray root, from its own path.
  ///
  /// The `.thm` is the sibling with the extension swapped, which is the same rule the engine resolves a descriptor by,
  /// and the only rule available where there is no tree to search. Nothing else is resolved: a bump name this
  /// descriptor declares is an engine reference that means nothing until somebody says which game data to read it
  /// against, and that is the editor's question rather than this one.
  ///
  /// # Errors
  ///
  /// Returns an error when the source names no file on disk, which for a standalone description is the whole address.
  fn describe_standalone(source: TextureSource, roots: XrayRoots) -> TauriResult<Self> {
    let path: PathBuf = source
      .physical_path()
      .ok_or_else(|| String::from("A texture outside every root has to be named by a file path"))?
      .to_path_buf();

    let texture_path: PathBuf = path.with_extension(to_extension(XrayAssetType::Dds));
    let descriptor_path: PathBuf = path.with_extension(to_extension(XrayAssetType::Thm));
    let texture: Option<XrayAsset> = texture_path.is_file().then(|| to_loose_asset(&texture_path)).flatten();

    // The roots this description is read back through, which have to include the folder the file is sitting in.
    // Everything downstream asks for bytes by logical path - the preview's decode, the lit surface's upload - and a
    // file no tree can place answers to its own name in its own directory or to nothing at all.
    let roots: XrayRoots = with_own_directory(roots, &texture_path);

    Ok(Self {
      form: read_descriptor_from_path(&descriptor_path)
        .as_ref()
        .map(TextureDescriptorForm::read),
      targets: Some(TextureEditTargets::of_paths(&descriptor_path, &texture_path)?),
      base: texture
        .as_ref()
        .and_then(|_| AssetTextureDescriptor::describe_path(&texture_path)),
      reference: source.to_label(),
      material: None,
      bump: None,
      companion: None,
      source,
      roots,
      texture,
    })
  }
}

/// The given roots with the file's own directory searched first.
///
/// Prepended rather than appended: the file the caller named is the one they mean, and a configured tree that happens
/// to hold a texture of the same name must not answer for it.
fn with_own_directory(roots: XrayRoots, texture_path: &Path) -> XrayRoots {
  match texture_path.parent() {
    Some(directory) => XrayRoots {
      asset: roots.asset,
      roots: std::iter::once(XrayRoot::new(directory.to_path_buf(), XrayMountMode::Directory))
        .chain(roots.roots)
        .collect(),
    },
    None => roots,
  }
}

/// The extension a kind of asset carries, without the leading dot `Path::with_extension` refuses.
fn to_extension(asset_type: XrayAssetType) -> &'static str {
  asset_type
    .get_rules()
    .map_or("", |rules| rules.extension.trim_start_matches('.'))
}

/// A loose file as the VFS would report it, rooted at its own directory.
///
/// Its logical path is the file name alone, which yields no engine reference - correctly, because there is none. What
/// this carries is the physical address, so everything that describes a located file works on it unchanged.
fn to_loose_asset(path: &Path) -> Option<XrayAsset> {
  let name: String = path.file_name()?.to_string_lossy().into_owned();

  Some(XrayAsset::new(
    XrayLogicalPath::new(&name).ok()?,
    XrayAssetContainer::Directory {
      root: path.parent()?.to_path_buf(),
      relative_path: PathBuf::from(name),
    },
  ))
}

/// Read a located `.thm`, or nothing when its bytes cannot be reached or are not a descriptor.
///
/// A second read of a file [`XrayMaterialResolver`] has already parsed, and deliberately so: the resolver answers what
/// the renderer makes of a descriptor and hands back no file, while the editor needs the chunks themselves. Folding
/// the two would put an editor's concern inside the crate that models the engine's reading of one.
fn read_descriptor(probe: &XrayProbe, asset: &XrayAsset) -> Option<ThmFile> {
  ThmFile::read_from_bytes::<XRayByteOrder>(probe.read_asset_bytes(asset).ok()?).ok()
}

/// Read a `.thm` straight off disk, for a file no mount holds.
fn read_descriptor_from_path(path: &Path) -> Option<ThmFile> {
  path
    .is_file()
    .then(|| ThmFile::read_from_path::<XRayByteOrder, _>(&path).ok())
    .flatten()
}
