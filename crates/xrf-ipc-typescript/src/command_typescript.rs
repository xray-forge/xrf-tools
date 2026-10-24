use std::path::Path;
use std::sync::{Arc, Mutex};

use specta_typescript::{Exporter, Typescript};
use tauri_specta::{BuilderConfiguration, LanguageExt};

use crate::collected_commands::CollectedCommands;
use crate::constants::GENERATED_HEADER;
use crate::generated_output::normalize_generated_bindings;
use crate::specta_command::SpectaCommand;

/// Exports one plugin's commands while recording every type those commands referenced, and the commands themselves.
///
/// Tauri Specta inlines the full transitive closure of a plugin's command signatures and cannot reference a
/// declaration living in another file. Collecting the types here lets each one be written exactly once, into
/// the module of the crate that declares it, and the inlined copies replaced by imports afterwards.
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
    normalize_generated_bindings(path)?;

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

    exporter.header(GENERATED_HEADER).framework_prelude("").into()
  }
}
