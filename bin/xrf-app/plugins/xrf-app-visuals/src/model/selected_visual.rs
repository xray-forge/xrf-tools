use std::collections::HashMap;
use std::sync::Arc;

use xrf_app_core::TauriResult;
use xrf_app_core::assets::{AssetMountState, AssetSource, AssetTextureDescriptor};
use xrf_chunk::XRayByteOrder;
use xrf_material::{
  XrayMaterialDescriptor, XrayMaterialResolver, XraySurfaceDescriptor, XraySurfaceResolver, XrayTextureScope,
};
use xrf_ogf::OgfFile;
use xrf_vfs::{XrayRoots, XraySearch};
use xrf_visual::{VisualDependencies, VisualDescription, VisualMotions, VisualPackage, VisualPacker};

use crate::SelectedVisualDescription;

/// One opened model: the visual packed for the renderer, what posing needs of it, and its description, made once at
/// the open and shared by every answer.
pub struct SelectedVisual {
  pub description: Arc<SelectedVisualDescription>,
  pub package: VisualPackage,
  /// What posing needs from the file, or `None` when the visual carries no bind pose.
  pub motions: Option<VisualMotions>,
}

impl SelectedVisual {
  /// Reads the visual `source` names within `roots`, centred on it where it is a file, through a fresh search, and
  /// resolves every reference it declares.
  ///
  /// # Errors
  ///
  /// Returns an error when the roots cannot be searched or the visual cannot be read.
  pub fn read(assets: &AssetMountState, source: AssetSource, roots: XrayRoots) -> TauriResult<Self> {
    let roots: XrayRoots = roots.centred_on(source.physical_path());
    let search: XraySearch = assets.search_fresh(&roots)?;
    // Read once and pack from what was read: the skeleton posing needs comes off the same parse, so keeping it costs no
    // second read of a file that may sit inside a volume.
    let file: Arc<OgfFile> = Self::read_file(&source, &search)?;
    let package: VisualPackage = VisualPacker::pack(&file);
    let dependencies: VisualDependencies = VisualDependencies::resolve(&package.description, &search);
    let description: SelectedVisualDescription = SelectedVisualDescription {
      textures: Self::describe_textures(&search, &dependencies),
      materials: Self::describe_materials(&search, &dependencies),
      surfaces: Self::describe_surfaces(&search, &package.description),
      textures_ltx: XrayMaterialResolver::find_textures_ltx(&search),
      description: package.description.clone(),
      dependencies,
      source,
      roots,
    };

    Ok(Self {
      description: Arc::new(description),
      motions: VisualMotions::of(&file),
      package,
    })
  }

  /// What the page shows of the model.
  pub fn describe(&self) -> Arc<SelectedVisualDescription> {
    Arc::clone(&self.description)
  }

  /// Reads a visual, whichever way its source names it: a file from disk, an asset served as it was parsed where the
  /// search's snapshot retains models.
  fn read_file(source: &AssetSource, search: &XraySearch) -> TauriResult<Arc<OgfFile>> {
    match source {
      AssetSource::File { path } => OgfFile::read_from_path::<XRayByteOrder, _>(path)
        .map(Arc::new)
        .map_err(|error| format!("Failed to read visual '{path}': {error}")),
      AssetSource::Asset { logical_path } => {
        let Some(asset) = search
          .find(logical_path)
          .map_err(|error| format!("Rejected visual '{logical_path}': {error}"))?
        else {
          return Err(format!(
            "Failed to read visual '{logical_path}': it resolves to nothing"
          ));
        };

        search
          .read_parsed(&asset, OgfFile::read_from_bytes::<XRayByteOrder>)
          .map_err(|error| format!("Failed to read visual '{logical_path}': {error}"))
      }
    }
  }

  /// Describes the file behind every located texture reference, once per file.
  fn describe_textures(
    search: &XraySearch,
    dependencies: &VisualDependencies,
  ) -> HashMap<String, AssetTextureDescriptor> {
    let mut described: HashMap<String, AssetTextureDescriptor> = HashMap::new();

    for texture in &dependencies.textures {
      for asset in texture.resolution.get_assets() {
        let path: &str = asset.get_logical_path().as_str();

        if described.contains_key(path) {
          continue;
        }

        if let Some(descriptor) = AssetTextureDescriptor::describe(search, asset) {
          described.insert(String::from(path), descriptor);
        }
      }
    }

    described
  }

  /// Describes the material the renderer builds for every declared texture reference, once per reference.
  fn describe_materials(
    search: &XraySearch,
    dependencies: &VisualDependencies,
  ) -> HashMap<String, XrayMaterialDescriptor> {
    let mut described: HashMap<String, XrayMaterialDescriptor> = HashMap::new();

    for texture in &dependencies.textures {
      if described.contains_key(&texture.reference) {
        continue;
      }

      described.insert(
        texture.reference.clone(),
        XrayMaterialResolver::describe_texture(search, &XrayTextureScope::shared(), &texture.reference),
      );
    }

    described
  }

  /// Describes how the renderer draws each submesh of the model, in the order the model declares them.
  fn describe_surfaces(search: &XraySearch, description: &VisualDescription) -> Vec<XraySurfaceDescriptor> {
    let resolver: XraySurfaceResolver = XraySurfaceResolver::open(search, XrayTextureScope::shared());

    description
      .submeshes
      .iter()
      .map(|submesh| {
        // A submesh dresses with the one texture it names, which is the list the engine compiles its shader against.
        let textures: Vec<String> = submesh.texture_name.clone().into_iter().collect();

        resolver.describe(submesh.shader_name.as_deref().unwrap_or_default(), &textures)
      })
      .collect()
  }
}
