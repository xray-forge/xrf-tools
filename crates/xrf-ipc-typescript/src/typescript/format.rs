use std::borrow::Cow;

use specta::datatype::{DataType, Primitive};
use specta::{Format, Type, Types};
use specta_util::Remapper;

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
