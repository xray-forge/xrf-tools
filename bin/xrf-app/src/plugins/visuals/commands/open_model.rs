use std::collections::HashMap;
use std::sync::Arc;

use tauri::State;
use xrf_material::{XrayMaterialDescriptor, XrayMaterialResolver, XraySurfaceDescriptor, XraySurfaceResolver};
use xrf_ogf::OgfFile;
use xrf_vfs::{XrayAsset, XrayProbe, XrayRoots};
use xrf_visual::{VisualDependencies, VisualDescription, VisualPackage, VisualPacker};

use crate::core::assets::{AssetMountState, AssetTextureDescriptor};
use crate::core::session::{Session, SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::visuals::read::read_source;
use crate::plugins::visuals::skeleton::SelectedSkeleton;
use crate::plugins::visuals::state::{SelectedVisual, SelectedVisualDescription, VisualSource, VisualState};

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
  state.selected.begin_open(session_id)?;

  log::info!("Opening visual: {}", source.label());

  let roots: XrayRoots = roots.centred_on(source.physical_path());

  let (package, dependencies, textures, materials, surfaces, textures_ltx, skeleton) =
    assets.with_probe(&roots, |probe| {
      // Read once and pack from what was read: the skeleton posing needs comes off the same parse, so keeping it costs
      // no second read of a file that may sit inside a volume.
      let file: OgfFile = read_source(&source, probe)?;
      let package: VisualPackage = VisualPacker::pack(&file);
      let dependencies: VisualDependencies = VisualDependencies::resolve(&package.description, probe);
      let textures: HashMap<String, AssetTextureDescriptor> = describe_textures(probe, &dependencies);
      let materials: HashMap<String, XrayMaterialDescriptor> = describe_materials(probe, &dependencies);
      let surfaces: Vec<XraySurfaceDescriptor> = describe_surfaces(probe, &package.description);
      let textures_ltx: Option<XrayAsset> = XrayMaterialResolver::find_textures_ltx(probe);

      TauriResult::Ok((
        package,
        dependencies,
        textures,
        materials,
        surfaces,
        textures_ltx,
        SelectedSkeleton::of(&file),
      ))
    })??;

  let selected: Arc<SessionSnapshot<SelectedVisual>> = state.selected.commit_open(
    session_id,
    SelectedVisual {
      source,
      roots,
      package,
      dependencies,
      skeleton,
      posed: Session::new("visual motion"),
      textures,
      materials,
      surfaces,
      textures_ltx,
    },
  )?;

  Ok(selected.map(SelectedVisual::describe))
}

/// Describes the file behind every located texture reference, once per file.
fn describe_textures(probe: &XrayProbe, dependencies: &VisualDependencies) -> HashMap<String, AssetTextureDescriptor> {
  let mut described: HashMap<String, AssetTextureDescriptor> = HashMap::new();

  for texture in &dependencies.textures {
    for asset in texture.resolution.get_assets() {
      let path: &str = asset.get_logical_path().as_str();

      if described.contains_key(path) {
        continue;
      }

      if let Some(descriptor) = AssetTextureDescriptor::describe(probe, asset) {
        described.insert(String::from(path), descriptor);
      }
    }
  }

  described
}

/// Describes the material the renderer builds for every declared texture reference, once per reference.
fn describe_materials(probe: &XrayProbe, dependencies: &VisualDependencies) -> HashMap<String, XrayMaterialDescriptor> {
  let mut described: HashMap<String, XrayMaterialDescriptor> = HashMap::new();

  for texture in &dependencies.textures {
    if described.contains_key(&texture.reference) {
      continue;
    }

    described.insert(
      texture.reference.clone(),
      XrayMaterialResolver::describe_texture(probe, &texture.reference),
    );
  }

  described
}

/// Describes how the renderer draws each submesh of the model, in the order the model declares them.
fn describe_surfaces(probe: &XrayProbe, description: &VisualDescription) -> Vec<XraySurfaceDescriptor> {
  let resolver: XraySurfaceResolver = XraySurfaceResolver::open(probe);

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
