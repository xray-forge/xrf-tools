use std::fmt::Debug;

use xrf_error::XrfResult;

use crate::dialect::LtxResolution;
use crate::source::LtxDocumentSource;

/// Which rules turn a config tree into resolved sections.
///
/// One implementation is standard LTX; another is the Monolith DLTX patch dialect in `xrf-dltx`. They are siblings
/// rather than a stack, because DLTX changes how base data resolves - includes rank by depth instead of read order,
/// inheritance resolves lazily - so it cannot take the standard resolver's output as input.
///
/// Held as a trait object and chosen once, where a flag or a setting is read. Everything downstream receives resolved
/// values without knowing which dialect produced them.
pub trait LtxDialect: Debug + Send + Sync {
  /// How this dialect names itself in output and diagnostics.
  fn get_name(&self) -> &'static str;

  /// Files that patch another config rather than standing on their own.
  ///
  /// Project assembly asks this so an attachment is not mistaken for an entry point: under DLTX a `mod_system_a.ltx`
  /// beside `system.ltx` is a patch of it, and verifying it alone would report every override as an orphan. Standard
  /// LTX has no such files and answers with nothing.
  ///
  /// # Errors
  ///
  /// Returns an error when the source cannot be listed.
  fn plan_attachments(&self, roots: &[String], source: &dyn LtxDocumentSource) -> XrfResult<Vec<String>>;

  /// Resolves one root into sections, under this dialect's rules.
  ///
  /// # Errors
  ///
  /// Returns an error for anything this dialect refuses, which for both current implementations means what the engine
  /// would refuse to start on.
  fn resolve(&self, root: &str, source: &dyn LtxDocumentSource) -> XrfResult<LtxResolution>;
}
