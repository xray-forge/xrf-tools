/// Extension of an engine string-table file, without a dot.
///
/// Kept beside its dotted form rather than derived from it, because the two path domains ask for it
/// differently: `Path::extension` answers without a dot, and `XrayLogicalPath::has_extension` matches
/// with one so that `notes.myxml` cannot pass as XML.
pub(crate) const FILE_EXTENSION: &str = "xml";

/// The same extension as a logical path carries it.
pub(crate) const FILE_EXTENSION_DOT: &str = ".xml";
