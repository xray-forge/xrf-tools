//! Multi-language JSON sources: the form the project authors translations in.

pub(crate) mod constants;
pub(crate) mod normalize;
pub(crate) mod read;
pub(crate) mod write;

pub(crate) use constants::FILE_EXTENSION;

#[cfg(test)]
mod tests;
