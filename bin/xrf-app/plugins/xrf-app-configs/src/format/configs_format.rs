use xrf_app_core::TauriResult;
use xrf_app_core::assets::AssetMountState;
use xrf_error::XrfError;
use xrf_ltx::{LtxFormatOptions, LtxProject, LtxProjectFormatResult};
use xrf_utils::error_to_string;
use xrf_vfs::{XrayRoots, XraySearch, XrayTransaction};

/// Formatting the configs a set of roots holds, or checking that they are formatted.
pub struct ConfigsFormat;

impl ConfigsFormat {
  /// Formats the configs `roots` hold below `prefix` and commits every rewrite as one change of the shared snapshot, all
  /// or none, so the cache, sessions and pages follow it as any save. The configs are read through a mount of the
  /// format's own, walked afresh, so one created behind the app is formatted too.
  ///
  /// # Errors
  ///
  /// Returns an error, with nothing written, when the roots cannot be mounted, a config is archived, cannot be read or
  /// will not parse, or the rewrite cannot be committed.
  pub fn format(
    assets: &AssetMountState,
    roots: &XrayRoots,
    prefix: Option<&str>,
    options: LtxFormatOptions,
  ) -> TauriResult<LtxProjectFormatResult> {
    // Opened in the shared snapshot too, so every rewrite lands in a mount it holds.
    let search: XraySearch = assets.search_fresh(roots)?;
    let project: LtxProject =
      LtxProject::open_at_roots_opt(roots, prefix, Default::default()).map_err(error_to_string)?;

    project
      .format_all_files_with(options, |rewrites| {
        let transaction: XrayTransaction = rewrites
          .into_iter()
          .fold(XrayTransaction::new(&search), |transaction, (file, bytes)| {
            transaction.with_file_write(file, bytes)
          });

        assets.commit(&transaction).map(drop).map_err(XrfError::new_asset_error)
      })
      .map_err(error_to_string)
  }

  /// Reports which configs `roots` hold below `prefix` are not formatted, writing nothing.
  ///
  /// # Errors
  ///
  /// Returns an error when the roots cannot be mounted, or a config cannot be read or will not parse.
  pub fn check(
    roots: &XrayRoots,
    prefix: Option<&str>,
    options: LtxFormatOptions,
  ) -> TauriResult<LtxProjectFormatResult> {
    LtxProject::open_at_roots_opt(roots, prefix, Default::default())
      .and_then(|project| project.check_format_all_files_opt(options))
      .map_err(error_to_string)
  }
}
