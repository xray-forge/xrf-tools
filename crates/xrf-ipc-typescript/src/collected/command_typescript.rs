use std::path::Path;
use std::sync::{Arc, Mutex};

use specta_typescript::{Exporter, Typescript};
use tauri_specta::{BuilderConfiguration, LanguageExt};

use crate::collected::{CollectedCommands, CommandWrappers};
use crate::write::GeneratedOutput;

/// Exports one plugin's commands while recording every type those commands referenced, and the commands themselves.
pub(crate) struct CommandTypescript {
  collected: Arc<Mutex<CollectedCommands>>,
}

impl LanguageExt for CommandTypescript {
  type Error = specta_typescript::Error;

  /// Records the plugin's types and its rendered commands; writes nothing, the command module being written once the
  /// type modules say what it imports.
  fn export(self, config: &BuilderConfiguration, _path: &Path) -> Result<(), Self::Error> {
    let commands = CommandWrappers::render(config)?;
    let mut collected = self.collected.lock().expect("Collected commands lock is poisoned");

    collected.types.extend(&config.types);
    collected
      .plugins
      .insert(config.plugin_name.unwrap_or_default().to_string(), commands);

    Ok(())
  }
}

impl CommandTypescript {
  /// An exporter that records what it renders into `collected`.
  pub(crate) fn collecting(collected: Arc<Mutex<CollectedCommands>>) -> Self {
    Self { collected }
  }

  /// The plain exporter both doors write through: this crate's header, and none of Specta's framework prelude.
  pub(crate) fn configured() -> Typescript {
    let exporter: Exporter = Typescript::default().into();

    exporter.header(GeneratedOutput::HEADER).framework_prelude("").into()
  }
}
