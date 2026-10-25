use std::sync::Arc;

use bytes::Bytes;
use xrf_app_core::TauriResult;
use xrf_app_core::assets::{AssetMountState, AssetSource};
use xrf_app_core::execution::ExecutionState;
use xrf_app_core::session::{SessionId, SessionSnapshot};
use xrf_dds::{DdsFile, DdsPng};
use xrf_dltx::DltxDialect;
use xrf_error::XrfResult;
use xrf_ltx::{Ltx, LtxProject, LtxProjectOptions};
use xrf_texture::EquipmentSlotOccupant;
use xrf_utils::error_to_string;
use xrf_vfs::{XrayAsset, XrayAssetType, XrayLogicalPath, XraySearch};

use crate::{
  EquipmentSheetLocation, EquipmentSheetSource, EquipmentSpriteMetadata, EquipmentSpriteOpen, EquipmentSprites,
};

/// One opened sheet: what the editor is told about it, and the picture it is drawn from.
pub struct EquipmentSpriteDocument {
  pub metadata: EquipmentSpriteMetadata,
  /// The sheet as a PNG, served to the page as it is held.
  pub preview: Bytes,
}

impl EquipmentSpriteDocument {
  /// Reads every part before any of it can become the opened sprite, the sheet and its configuration through one fresh
  /// search of the roots where either lies in them.
  ///
  /// # Errors
  ///
  /// Returns an error where the roots cannot be searched or the sheet cannot be read or decoded; a configuration that
  /// cannot be read is kept as the document's `config_error` instead.
  pub fn read(assets: &AssetMountState, open: EquipmentSpriteOpen) -> TauriResult<Self> {
    let search: Option<XraySearch> = open
      .reads_roots()
      .then(|| assets.search_fresh(&open.roots))
      .transpose()?;
    let (preview, location): (DdsPng, EquipmentSheetLocation) = Self::read_sheet(search.as_ref(), &open.sheet)?;
    let (occupants, config_error): (Vec<EquipmentSlotOccupant>, Option<String>) =
      match Self::read_occupants(search.as_ref(), &open) {
        Ok(occupants) => (occupants, None),
        Err(error) => (Vec::new(), Some(error)),
      };

    Ok(Self {
      metadata: EquipmentSpriteMetadata {
        config_error,
        location,
        occupants,
        open,
      },
      preview: Bytes::from(preview.bytes),
    })
  }

  /// Reads the sheet `previous` holds again, as it was asked for, into the new handle `id` on the pool; a read that
  /// fails gives the new handle up and leaves `previous` as it was.
  ///
  /// # Errors
  ///
  /// Returns an error where `previous` holds nothing, `id` cannot be reserved, or the sheet cannot be read again.
  pub async fn reload(
    sprites: &EquipmentSprites,
    execution: &ExecutionState,
    assets: &AssetMountState,
    id: SessionId,
    previous: SessionId,
  ) -> TauriResult<Arc<SessionSnapshot<Self>>> {
    let assets: AssetMountState = AssetMountState::clone(assets);

    sprites
      .reload_on_pool(execution, id, previous, "Reloading an equipment sheet", move |held| {
        log::info!("Reloading equipment sheet: {}", held.metadata.open.sheet.label());

        Self::read(&assets, held.metadata.open.clone())
      })
      .await
  }

  /// Decodes the sheet and works out where it came from.
  fn read_sheet(
    search: Option<&XraySearch>,
    source: &EquipmentSheetSource,
  ) -> TauriResult<(DdsPng, EquipmentSheetLocation)> {
    match source {
      EquipmentSheetSource::File { path } => Ok((
        Self::to_preview(DdsFile::read_from_path(path))?,
        EquipmentSheetLocation::of_file(path),
      )),
      EquipmentSheetSource::Asset { reference } => Self::read_located_sheet(Self::require(search)?, reference),
    }
  }

  /// Decodes the sheet a reference resolves to in the mounted roots.
  fn read_located_sheet(search: &XraySearch, reference: &str) -> TauriResult<(DdsPng, EquipmentSheetLocation)> {
    let asset: XrayAsset = search
      .resolve(XrayAssetType::Dds, reference)
      .map_err(|error| format!("Rejected equipment sheet '{reference}': {error}"))?
      .get_asset()
      .cloned()
      .ok_or_else(|| format!("Equipment sheet '{reference}' resolves to nothing in the mounted roots"))?;

    let preview: DdsPng = Self::to_preview(
      search
        .read_asset(&asset)
        .and_then(|bytes| DdsFile::read_from_bytes(&bytes)),
    )?;

    Ok((preview, EquipmentSheetLocation::of_asset(asset)))
  }

  /// Decodes whatever the sheet turned out to be.
  fn to_preview(read: XrfResult<DdsFile>) -> TauriResult<DdsPng> {
    read.and_then(|dds| dds.to_png()).map_err(error_to_string)
  }

  /// Reads the configuration and lists what it puts on the sheet.
  fn read_occupants(
    search: Option<&XraySearch>,
    open: &EquipmentSpriteOpen,
  ) -> Result<Vec<EquipmentSlotOccupant>, String> {
    let Some(config) = open.config.as_ref() else {
      return Ok(Vec::new());
    };

    Self::read_config(search, open, config)
      .map(|ltx| EquipmentSlotOccupant::new_list_from_ltx(&ltx))
      .map_err(|error| format!("Could not read '{}': {error}", config.label()))
  }

  /// Resolves one configuration: a file from its own directory, an entry point through the roots' search, so it reads
  /// the same snapshot as the sheet.
  fn read_config(search: Option<&XraySearch>, open: &EquipmentSpriteOpen, config: &AssetSource) -> TauriResult<Ltx> {
    match config {
      AssetSource::File { path } => {
        Ltx::read_from_file_with_dialect(path, DltxDialect::select(open.is_dltx).as_ref()).map_err(error_to_string)
      }
      AssetSource::Asset { logical_path } => {
        let options: LtxProjectOptions = LtxProjectOptions::default().with_dialect(DltxDialect::select(open.is_dltx));
        let project: LtxProject =
          LtxProject::open_at_search_opt(open.roots.describe(), Self::require(search)?, None, options)
            .map_err(error_to_string)?;
        let entry: XrayLogicalPath = XrayLogicalPath::new(logical_path).map_err(error_to_string)?;

        project
          .read_resolution(&entry)
          .map(|resolution| resolution.ltx.clone())
          .map_err(error_to_string)
      }
    }
  }

  fn require(search: Option<&XraySearch>) -> TauriResult<&XraySearch> {
    search.ok_or_else(|| String::from("The equipment sheet's roots were not searched"))
  }
}
