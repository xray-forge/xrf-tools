#![doc = include_str!("../README.md")]

pub(crate) mod efd_data_format;
pub(crate) mod efd_file;
pub(crate) mod efd_pattern;
pub(crate) mod efd_variable;

#[cfg(test)]
mod tests;

pub use crate::efd_data_format::EfdDataFormat;
pub use crate::efd_file::EfdFile;
pub use crate::efd_pattern::EfdPattern;
pub use crate::efd_variable::EfdVariable;
