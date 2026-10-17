use xrf_error::XrfResult;

use crate::host::render_asset_source::RenderAssetSource;

/// A source holding no textures: every request still takes a slot, which loads as missing.
pub struct EmptySource;

impl RenderAssetSource for EmptySource {
  fn get_texture_scope(&self) -> &str {
    "test"
  }

  fn read_texture(&self, _: &str) -> XrfResult<Option<Vec<u8>>> {
    Ok(None)
  }
}
