use xrf_archive::{DESCRIPTOR_ROW_FIELDS_SIZE, DESCRIPTOR_ROW_SIZE_FIELD_SIZE};
use xrf_error::{XrfError, XrfResult};
use xrf_utils::encode_string_to_w1251_bytes;

/// A validated Windows-1251 name and its descriptor row size.
pub(crate) struct DescriptorName {
  pub(super) encoded: Vec<u8>,
  /// The row's leading field: the bytes that follow it, which is not the whole row.
  pub(super) declared_size: u16,
}

impl DescriptorName {
  /// Encode a name as the windows-1251 the engine reads, refusing one no descriptor row can declare.
  pub(crate) fn encode(name: &str) -> XrfResult<Self> {
    let encoded: Vec<u8> = encode_string_to_w1251_bytes(name).map_err(|error| {
      XrfError::new_encoding_error(format!(
        "File name '{name}' cannot be written as windows-1251, which the engine requires: {error}"
      ))
    })?;

    let declared_size: u16 = u16::try_from(encoded.len() + usize::from(DESCRIPTOR_ROW_FIELDS_SIZE))
      .map_err(|_| XrfError::new_invalid_error(format!("File name '{name}' is too long for a descriptor row")))?;

    Ok(Self { encoded, declared_size })
  }

  /// Bytes this name's row costs the table it joins, its leading size field included.
  pub(crate) fn get_row_size(&self) -> u64 {
    u64::from(DESCRIPTOR_ROW_SIZE_FIELD_SIZE) + u64::from(self.declared_size)
  }
}

#[cfg(test)]
mod tests {
  use super::DescriptorName;

  #[test]
  fn a_cyrillic_name_costs_one_byte_per_character() {
    // Windows-1251 is single byte, so the row is shorter than the UTF-8 the name arrived as.
    let name: DescriptorName = DescriptorName::encode("configs\\текст.ltx").expect("name encodes");

    assert_eq!(name.get_row_size(), "configs\\текст.ltx".chars().count() as u64 + 18);
  }

  #[test]
  fn a_name_the_engine_could_not_read_never_reaches_a_row() {
    assert!(DescriptorName::encode("configs\\ロゴ.ltx").is_err());
  }
}
