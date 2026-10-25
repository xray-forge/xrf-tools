use std::collections::BTreeMap;

use specta::Types;

use crate::collected::SpectaCommand;

/// What exporting every plugin collected: the types their commands referenced, and each plugin's commands.
#[derive(Default)]
pub(crate) struct CollectedCommands {
  pub types: Types,
  /// Plugin name to its commands, in the order the builder registered them.
  pub commands: BTreeMap<String, Vec<SpectaCommand>>,
}

impl CollectedCommands {
  /// The commands `plugin` exported.
  ///
  /// # Panics
  ///
  /// Panics when the plugin exported nothing, which means its builder carries another plugin name than its surface.
  pub(crate) fn commands_of(&self, plugin: &str) -> &[SpectaCommand] {
    self
      .commands
      .get(plugin)
      .unwrap_or_else(|| panic!("Plugin `{plugin}` exported no commands under its own name"))
  }
}
