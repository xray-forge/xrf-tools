use xrf_dds::{DdsFile, DdsPng};
use xrf_dltx::select_ltx_dialect;
use xrf_error::XrfResult;
use xrf_ltx::{Ltx, LtxProject, LtxProjectOptions};
use xrf_texture::EquipmentSlotOccupant;
use xrf_vfs::{XrayAsset, XrayAssetType, XrayLogicalPath, XrayProbe, XrayRoots};

use crate::core::assets::AssetMountState;
use crate::core::error::error_to_string;
use crate::core::ltx::open_ltx_project;
use crate::core::types::TauriResult;
use crate::plugins::sprite_equipment::location::EquipmentSheetLocation;
use crate::plugins::sprite_equipment::metadata::EquipmentSpriteMetadata;
use crate::plugins::sprite_equipment::source::{EquipmentConfigSource, EquipmentSheetSource, EquipmentSpriteOpen};

/// One opened sheet: what the editor is told about it, and the bytes it is drawn from.
///
/// Metadata and bytes belong to one immutable publication, so a reload that fails replaces neither.
pub struct EquipmentSpriteDocument {
  pub metadata: EquipmentSpriteMetadata,
  pub preview: Vec<u8>,
}

impl EquipmentSpriteDocument {
  /// Name the sheet is streamed to the webview under, which is the same for every open.
  const PREVIEW_NAME: &'static str = "equipment.png";

  /// Reads every part before any of it can become the opened sprite.
  pub fn read(assets: &AssetMountState, open: EquipmentSpriteOpen) -> TauriResult<Self> {
    let (preview, location): (DdsPng, EquipmentSheetLocation) = Self::read_sheet(assets, &open.roots, &open.sheet)?;
    let (occupants, config_error): (Vec<EquipmentSlotOccupant>, Option<String>) = match Self::read_occupants(&open) {
      Ok(occupants) => (occupants, None),
      Err(error) => (Vec::new(), Some(error)),
    };

    Ok(Self {
      metadata: EquipmentSpriteMetadata {
        name: Self::PREVIEW_NAME.into(),
        config_error,
        location,
        occupants,
        open,
      },
      preview: preview.bytes,
    })
  }

  /// Decodes the sheet and works out where it came from.
  fn read_sheet(
    assets: &AssetMountState,
    roots: &XrayRoots,
    source: &EquipmentSheetSource,
  ) -> TauriResult<(DdsPng, EquipmentSheetLocation)> {
    match source {
      EquipmentSheetSource::File { path } => Ok((
        Self::to_preview(DdsFile::read_from_path(path))?,
        EquipmentSheetLocation::of_file(path),
      )),
      EquipmentSheetSource::Asset { reference } => {
        assets.with_probe(roots, |probe| Self::read_located_sheet(probe, reference))?
      }
    }
  }

  /// Decodes the sheet a reference resolves to in the mounted roots.
  fn read_located_sheet(probe: &XrayProbe, reference: &str) -> TauriResult<(DdsPng, EquipmentSheetLocation)> {
    let asset: XrayAsset = probe
      .resolve(XrayAssetType::Dds, reference)
      .map_err(|error| format!("Rejected equipment sheet '{reference}': {error}"))?
      .get_asset()
      .cloned()
      .ok_or_else(|| format!("Equipment sheet '{reference}' resolves to nothing in the mounted roots"))?;

    let preview: DdsPng = Self::to_preview(
      probe
        .read_asset_bytes(&asset)
        .and_then(|bytes| DdsFile::read_from_bytes(&bytes)),
    )?;

    Ok((preview, EquipmentSheetLocation::of_asset(asset)))
  }

  /// Decodes whatever the sheet turned out to be.
  fn to_preview(read: XrfResult<DdsFile>) -> TauriResult<DdsPng> {
    read.and_then(|dds| dds.to_png()).map_err(error_to_string)
  }

  /// Reads the configuration and lists what it puts on the sheet.
  ///
  /// A configuration nobody asked for is no occupants and no complaint, which is why the absent case is `Ok` rather
  /// than an error nothing went wrong to cause.
  fn read_occupants(open: &EquipmentSpriteOpen) -> Result<Vec<EquipmentSlotOccupant>, String> {
    let Some(config) = open.config.as_ref() else {
      return Ok(Vec::new());
    };

    Self::read_config(&open.roots, config, open.is_dltx)
      .map(|ltx| EquipmentSlotOccupant::new_list_from_ltx(&ltx))
      .map_err(|error| format!("Could not read '{}': {error}", config.label()))
  }

  /// Resolves one configuration, through the roots or from its own directory.
  fn read_config(roots: &XrayRoots, config: &EquipmentConfigSource, is_dltx: bool) -> TauriResult<Ltx> {
    match config {
      EquipmentConfigSource::File { path } => {
        Ltx::read_from_file_with_dialect(path, select_ltx_dialect(is_dltx).as_ref()).map_err(error_to_string)
      }
      EquipmentConfigSource::Asset { logical_path } => {
        let options: LtxProjectOptions = LtxProjectOptions::default().with_dialect(select_ltx_dialect(is_dltx));
        let project: LtxProject = open_ltx_project(roots, None, options).map_err(error_to_string)?;
        let entry: XrayLogicalPath = XrayLogicalPath::new(logical_path).map_err(error_to_string)?;

        project
          .read_resolution(&entry)
          .map(|resolution| resolution.ltx.clone())
          .map_err(error_to_string)
      }
    }
  }
}
