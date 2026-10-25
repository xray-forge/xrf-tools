use specta::datatype::DataType;
use specta::{Type, Types};

/// A constant's type as Specta defines it, read off the value so the expansion names no type the marked item spelled;
/// registers every named type that one references into `types`.
pub fn data_type_of<T: Type + ?Sized>(_value: &T, types: &mut Types) -> DataType {
  T::definition(types)
}

#[cfg(test)]
mod tests {
  use specta::Types;
  use specta::datatype::{DataType, Primitive};

  use crate::data_type_of;

  #[test]
  fn reads_the_type_off_the_value() {
    assert!(matches!(
      data_type_of(&4_u32, &mut Types::default()),
      DataType::Primitive(Primitive::u32)
    ));
  }
}
