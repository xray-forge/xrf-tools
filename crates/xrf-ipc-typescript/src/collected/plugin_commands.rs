use crate::collected::SpectaCommand;

/// One plugin's typed commands as the export collected them: each command, and the object of their wrappers.
#[derive(Clone, Debug)]
pub(crate) struct PluginCommands {
  /// In the order the builder registered them.
  pub commands: Vec<SpectaCommand>,
  /// The wrappers object, `{ ... }`, each wrapper invoking its command by the plugin's enum member.
  pub wrappers: String,
  /// Whether a wrapper takes or answers a Tauri `Channel`, which the module then imports.
  pub uses_channel: bool,
}
