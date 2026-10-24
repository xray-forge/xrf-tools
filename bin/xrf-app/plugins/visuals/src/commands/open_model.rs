use std::collections::HashMap;
use std::sync::Arc;

use tauri::State;
use xrf_app_core::assets::{AssetMountState, AssetTextureDescriptor};
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_app_core::types::TauriResult;
use xrf_material::{
  XrayMaterialDescriptor, XrayMaterialResolver, XraySurfaceDescriptor, XraySurfaceResolver, XrayTextureScope,
};
use xrf_ogf::OgfFile;
use xrf_vfs::{XrayAsset, XrayRoots, XraySearch};
use xrf_visual::{VisualDependencies, VisualDescription, VisualMotions, VisualPackage, VisualPacker};

use crate::read::read_source;
use crate::state::{SelectedVisual, SelectedVisualDescription, VisualSource, VisualState};

/// Select a visual and return what it contains, with every reference it declares resolved.
#[cfg_attr(feature = "typescript-bindings", specta::specta(rename = "open_model"))]
#[tauri::command(rename = "open_model")]
pub async fn visuals_open_model(
  session_id: SessionId,
  source: VisualSource,
  roots: XrayRoots,
  state: State<'_, VisualState>,
  assets: State<'_, AssetMountState>,
) -> TauriResult<SessionSnapshot<SelectedVisualDescription>> {
  state.models.begin_open(session_id)?;

  log::info!("Opening visual: {}", source.label());

  let roots: XrayRoots = roots.centred_on(source.physical_path());

  let search: XraySearch = assets.search(&roots)?;
  // Read once and pack from what was read: the skeleton posing needs comes off the same parse, so keeping it costs no
  // second read of a file that may sit inside a volume.
  let file: Arc<OgfFile> = read_source(&source, &search)?;
  let package: VisualPackage = VisualPacker::pack(&file);
  let dependencies: VisualDependencies = VisualDependencies::resolve(&package.description, &search);
  let textures: HashMap<String, AssetTextureDescriptor> = describe_textures(&search, &dependencies);
  let materials: HashMap<String, XrayMaterialDescriptor> = describe_materials(&search, &dependencies);
  let surfaces: Vec<XraySurfaceDescriptor> = describe_surfaces(&search, &package.description);
  let textures_ltx: Option<XrayAsset> = XrayMaterialResolver::find_textures_ltx(&search);
  let motions: Option<VisualMotions> = VisualMotions::of(&file);

  let selected: Arc<SessionSnapshot<SelectedVisual>> = state.models.commit_open(
    session_id,
    SelectedVisual {
      source,
      roots,
      package,
      dependencies,
      motions,
      textures,
      materials,
      surfaces,
      textures_ltx,
    },
  )?;

  Ok(selected.map(SelectedVisual::describe))
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
