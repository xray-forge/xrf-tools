use std::borrow::Cow;
use std::path::Path;
use std::sync::{Arc, Mutex};

use specta::datatype::{DataType, Primitive};
use specta::{Format, Type, Types};
use specta_typescript::{Exporter, Typescript};
use specta_util::Remapper;
use tauri_specta::{BuilderConfiguration, LanguageExt};

use crate::constants::GENERATED_HEADER;
use crate::output::normalize_generated_bindings;

/// Exports one plugin's commands while recording every type those commands referenced.
///
/// Tauri Specta inlines the full transitive closure of a plugin's command signatures and cannot reference a
/// declaration living in another file. Collecting the types here lets each one be written exactly once, into
/// the module of the crate that declares it, and the inlined copies replaced by imports afterwards.
pub struct CommandTypescript {
  exporter: Typescript,
  collected: Arc<Mutex<Types>>,
}

impl LanguageExt for CommandTypescript {
  type Error = specta_typescript::Error;

  fn export(self, config: &BuilderConfiguration, path: &Path) -> Result<(), Self::Error> {
    self
      .collected
      .lock()
      .expect("Collected types lock is poisoned")
      .extend(&config.types);

    LanguageExt::export(self.exporter, config, path)?;
    normalize_generated_bindings(path)?;

    Ok(())
  }
}

/// Serde shape plus the numeric remapping the frontend expects.
///
/// The builders set `dangerously_cast_bigints_to_number`, so the wide integer rules here are what keep types
/// rendered outside a command module identical to the ones rendered inside one.
#[derive(Debug, Clone)]
pub(crate) struct TypeScriptFormat {
  remapper: Remapper,
}

impl Default for TypeScriptFormat {
  fn default() -> Self {
    let number = <specta_typescript::Number as Type>::definition(&mut Types::default());
    let remapper = Remapper::new()
      .rule(DataType::Primitive(Primitive::usize), number.clone())
      .rule(DataType::Primitive(Primitive::isize), number.clone())
      .rule(DataType::Primitive(Primitive::u64), number.clone())
      .rule(DataType::Primitive(Primitive::i64), number.clone())
      .rule(DataType::Primitive(Primitive::u128), number.clone())
      .rule(DataType::Primitive(Primitive::i128), number.clone())
      .rule(
        <specta_typescript::BigInt as Type>::definition(&mut Types::default()),
        number,
      );

    Self { remapper }
  }
}

impl Format for TypeScriptFormat {
  fn map_types(&self, types: &Types) -> Result<Cow<'_, Types>, specta::FormatError> {
    let types = specta_serde::Format.map_types(types)?;

    Ok(Cow::Owned(self.remapper.remap_types(types.into_owned())))
  }

  fn map_type(&'_ self, types: &Types, data_type: &DataType) -> Result<Cow<'_, DataType>, specta::FormatError> {
    let data_type = specta_serde::Format.map_type(types, data_type)?;

    Ok(Cow::Owned(self.remapper.remap_dt(data_type.into_owned())))
  }
}

pub(crate) fn exporter() -> Typescript {
  let exporter: Exporter = Typescript::default().into();

  exporter.header(GENERATED_HEADER).framework_prelude("").into()
}

pub fn command_exporter(collected: Arc<Mutex<Types>>) -> CommandTypescript {
  CommandTypescript {
    exporter: exporter(),
    collected,
  }
}
