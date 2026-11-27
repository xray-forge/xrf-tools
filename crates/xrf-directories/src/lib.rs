#![doc = include_str!("../README.md")]

pub(crate) mod data_directories;
#[cfg(test)]
mod tests;

pub use crate::data_directories::DataDirectories;
