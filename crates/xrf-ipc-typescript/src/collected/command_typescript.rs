use std::path::Path;
use std::sync::{Arc, Mutex};

use specta_typescript::{Exporter, Typescript};
use tauri_specta::{BuilderConfiguration, LanguageExt};

use crate::collected::{CollectedCommands, SpectaCommand};
use crate::write::GeneratedOutput;

/// Exports one plugin's commands while recording every type those commands referenced, and the commands themselves.
pub(crate) struct CommandTypescript {
  exporter: Typescript,
  collected: Arc<Mutex<CollectedCommands>>,
}

impl LanguageExt for CommandTypescript {
  type Error = specta_typescript::Error;

  fn export(self, config: &BuilderConfiguration, path: &Path) -> Result<(), Self::Error> {
    {
      let mut collected = self.collected.lock().expect("Collected commands lock is poisoned");

      collected.types.extend(&config.types);
      collected.commands.insert(
        config.plugin_name.unwrap_or_default().to_string(),
        config.commands.iter().map(SpectaCommand::from).collect(),
      );
    }

    LanguageExt::export(self.exporter, config, path)?;
    GeneratedOutput::normalize_specta_module(path)?;

    Ok(())
  }
}

impl CommandTypescript {
  /// An exporter that records what it renders into `collected`.
  pub(crate) fn collecting(collected: Arc<Mutex<CollectedCommands>>) -> Self {
    Self {
      exporter: Self::configured(),
      collected,
    }
  }

  /// The plain exporter both doors write through: this crate's header, and none of Specta's framework prelude.
  pub(crate) fn configured() -> Typescript {
    let exporter: Exporter = Typescript::default().into();

    exporter.header(GeneratedOutput::HEADER).framework_prelude("").into()
  }
}
