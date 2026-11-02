//! `fsgame.ltx` and its Enhanced Edition namesakes: the file an installation describes its own directory layout with,
//! parsed and resolved to host paths.

mod fsgame_declaration;
mod fsgame_file;
#[cfg(test)]
mod tests;

pub use fsgame_declaration::FsgameDeclaration;
pub use fsgame_file::FsgameFile;
