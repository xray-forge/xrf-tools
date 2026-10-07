//! `fsgame.ltx` and its Enhanced Edition namesakes: the file an installation describes its own directory layout with,
//! parsed and resolved to host paths.

mod declaration;
mod file;
#[cfg(test)]
mod tests;

pub use declaration::FsgameDeclaration;
pub use file::FsgameFile;
