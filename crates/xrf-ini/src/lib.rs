#![doc = include_str!("../README.md")]

pub(crate) mod ini_encoding;
pub(crate) mod ini_field;
pub(crate) mod ini_file;
pub(crate) mod ini_format;
pub(crate) mod ini_override;
pub(crate) mod ini_repeated_key;
pub(crate) mod ini_section;

#[cfg(test)]
mod tests;

pub use crate::ini_encoding::IniEncoding;
pub use crate::ini_field::IniField;
pub use crate::ini_file::IniFile;
pub use crate::ini_format::IniFormat;
pub use crate::ini_override::IniOverride;
pub use crate::ini_repeated_key::IniRepeatedKey;
pub use crate::ini_section::IniSection;
