use std::collections::BTreeMap;

use specta::Types;
use xrf_error::{XrfError, XrfResult};

use crate::collected::PluginCommands;

/// What exporting every plugin collected: the types their commands referenced, and each plugin's commands.
#[derive(Default)]
pub(crate) struct CollectedCommands {
  pub types: Types,
  /// Plugin name to its commands.
  pub plugins: BTreeMap<String, PluginCommands>,
}

impl CollectedCommands {
  /// The commands `plugin` exported.
  ///
  /// # Errors
  ///
  /// When the plugin exported nothing, which means its builder carries another plugin name than its surface.
  pub(crate) fn of(&self, plugin: &str) -> XrfResult<&PluginCommands> {
    self
      .plugins
      .get(plugin)
      .ok_or_else(|| XrfError::new_invalid_error(format!("Plugin `{plugin}` exported no commands under its own name")))
  }
}
