/// Directory the engine excludes from languages by name rather than by content.
///
/// `CStringTable::FillLanguageToken` skips it explicitly, so a tool that reads languages by listing
/// directories has to skip it too or it invents a language the game does not have.
pub(crate) const MAP_DESC_DIRECTORY: &str = "map_desc";

/// A language directory holding only this is not a language, by the engine's own rule.
pub(crate) const OPENXRAY_XML: &str = "openxray.xml";
