//! `fsgame.ltx` and its Enhanced Edition namesakes: the file an installation describes its own directory layout with,
//! parsed and resolved to host paths; and the SDK's `fs.ltx`, the same format laying out rawdata beside gamedata.

mod fsgame_declaration;
mod fsgame_file;
#[cfg(test)]
mod tests;
mod xray_sdk_installation;

pub use fsgame_declaration::FsgameDeclaration;
pub use fsgame_file::FsgameFile;
pub use xray_sdk_installation::XraySdkInstallation;
