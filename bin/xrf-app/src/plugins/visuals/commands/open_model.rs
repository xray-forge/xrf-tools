use std::collections::HashMap;
use std::sync::MutexGuard;

use tauri::State;
use xrf_vfs::{XrayProbe, XrayRoots};
use xrf_db::OgfFile;
use xrf_visual::{VisualDependencies, VisualPackage, VisualPacker};

use crate::core::assets::{AssetMountState, AssetTextureDescriptor};
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
  source: VisualSource,
  roots: XrayRoots,
  state: State<'_, VisualState>,
  assets: State<'_, AssetMountState>,
) -> TauriResult<SelectedVisualDescription> {
  log::info!("Opening visual: {}", source.label());

  // Centred on the model unless the caller centred it elsewhere, and the effective roots is what travels back: a texture
  // resolved through the model's own tree has to be readable through the same tree afterwards.
  let roots: XrayRoots = roots.centred_on(source.physical_path());

  // Read, resolve and describe inside one probe, so the model, its references and the files behind them are all looked
  // for in the same roots: a second probe could mount a source between the calls and answer differently.
  let (package, dependencies, textures, skeleton) = assets.with_probe(&roots, |probe| {
    // Read once and pack from what was read: the skeleton posing needs comes off the same parse, so keeping it costs
    // no second read of a file that may sit inside a volume.
    let file: OgfFile = read_source(&source, probe)?;
    let package: VisualPackage = VisualPacker::pack(&file);
    let dependencies: VisualDependencies = VisualDependencies::resolve(&package.description, probe);
    let textures: HashMap<String, AssetTextureDescriptor> = describe_textures(probe, &dependencies);

    TauriResult::Ok((package, dependencies, textures, SelectedSkeleton::of(&file)))
  })??;

  let description: SelectedVisualDescription = SelectedVisualDescription {
    source: source.clone(),
    roots: roots.clone(),
    description: package.description.clone(),
    dependencies: dependencies.clone(),
    textures: textures.clone(),
  };

  let mut selected: MutexGuard<Option<SelectedVisual>> = state
    .selected
    .lock()
    .map_err(|error| format!("Failed to open visual - selection state is unavailable: {error}"))?;

  *selected = Some(SelectedVisual {
    source,
    roots,
    package,
    dependencies,
    skeleton,
    posed: None,
    textures,
  });

  Ok(description)
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
