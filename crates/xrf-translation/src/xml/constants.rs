/// Extension of an engine string-table file, without a dot.
///
/// Undotted because that is the one spelling both path domains now ask for: `Path::extension` answers without a dot,
/// and so does `XrayLogicalPath::has_extension`, which matches the split extension rather than a byte suffix — so
/// `notes.myxml` still cannot pass as XML.
pub(crate) const FILE_EXTENSION: &str = "xml";
