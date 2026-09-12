use std::collections::HashMap;
use std::sync::Arc;

use tauri::State;
use xrf_db::OgfFile;
use xrf_material::{XrayMaterialDescriptor, XrayMaterialResolver, XraySurfaceDescriptor, XraySurfaceResolver};
use xrf_vfs::{XrayAsset, XrayProbe, XrayRoots};
use xrf_visual::{VisualDependencies, VisualDescription, VisualPackage, VisualPacker};

use crate::core::assets::{AssetMountState, AssetTextureDescriptor};
use crate::core::session::{Session, SessionId, SessionSnapshot};
use crate::core::types::TauriResult;
use crate::plugins::visuals::read::read_source;
use crate::plugins::visuals::skeleton::SelectedSkeleton;
use crate::plugins::visuals::state::{SelectedVisual, SelectedVisualDescription, VisualSource, VisualState};

/// Select a visual and return what it contains, with every reference it declares resolved.
///
/// Geometry is packed here and parked, so the `read_geometry` that follows serves the same parse rather than repeating
/// it. The bytes are not returned: a typed command cannot carry them, which is why they are read separately.
///
/// Resolution happens once, for the whole dependency set, in this one call. That is what keeps a model with forty
/// textures from costing forty round trips, and it is why the outcomes travel with the description rather than being
/// asked for afterwards.
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

  // Centred on the model unless the caller centred it elsewhere, and the effective roots is what travels back: a texture
  // resolved through the model's own tree has to be readable through the same tree afterwards.
  let roots: XrayRoots = roots.centred_on(source.physical_path());

  // Read, resolve and describe inside one probe, so the model, its references and the files behind them are all looked
  // for in the same roots: a second probe could mount a source between the calls and answer differently.
  let (package, dependencies, textures, materials, surfaces, textures_ltx, skeleton) =
    assets.with_probe(&roots, |probe| {
      // Read once and pack from what was read: the skeleton posing needs comes off the same parse, so keeping it costs
      // no second read of a file that may sit inside a volume.
      let file: OgfFile = read_source(&source, probe)?;
      let package: VisualPackage = VisualPacker::pack(&file);
      let dependencies: VisualDependencies = VisualDependencies::resolve(&package.description, probe);
      let textures: HashMap<String, AssetTextureDescriptor> = describe_textures(probe, &dependencies);
      let materials: HashMap<String, XrayMaterialDescriptor> = describe_materials(probe, &dependencies);
      let surfaces: HashMap<String, XraySurfaceDescriptor> = describe_surfaces(probe, &package.description);
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
///
/// Keyed by logical path, which deduplicates as it goes: 6.5% of measured models point two submeshes at one texture, and
/// describing it twice would make a model's texture total the weight of its references rather than of its files.
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
///
/// Keyed by the reference rather than by the located file, and iterated over every dependency rather than only the
/// located ones, because the engine reads `<reference>.thm` whether or not `<reference>.dds` exists: a missing base
/// texture with a live bump declaration binds that bump over the dummy diffuse, which is exactly what a modder opening
/// the panel is trying to see.
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

/// Describes how the renderer draws every shader the model's submeshes declare, once per shader name.
///
/// The library is opened once for the whole model rather than per submesh: one `shaders.xr` answers every surface of
/// every model in a tree, and it is a 190KB chunked file.
///
/// Keyed by the shader name as the mesh spells it, which is how the engine looks a blender up, so the frontend joins
/// with `surfaces[submesh.shaderName]`. A submesh declaring no shader has no entry, which is the normal case for a
/// skeleton's own record.
fn describe_surfaces(probe: &XrayProbe, description: &VisualDescription) -> HashMap<String, XraySurfaceDescriptor> {
  let resolver: XraySurfaceResolver = XraySurfaceResolver::open(probe);
  let mut described: HashMap<String, XraySurfaceDescriptor> = HashMap::new();

  for submesh in &description.submeshes {
    let Some(shader) = submesh.shader_name.as_ref() else {
      continue;
    };

    if described.contains_key(shader) {
      continue;
    }

    described.insert(shader.clone(), resolver.describe(shader));
  }

  described
}
