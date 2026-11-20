pub(crate) const ROOT_SECTION: &str = "";

pub const LTX_SCHEME_FIELD: &str = "$scheme";

pub(crate) const LTX_SCHEME_STRICT_FIELD: &str = "$strict";

pub(crate) const LTX_SCHEME_LTX_FILENAME: &str = "scheme.ltx";

/// Root config the engine loads, which every project entry point chain reaches.
pub(crate) const SYSTEM_LTX_FILENAME: &str = "system.ltx";

pub(crate) const LTX_SCHEME_EXTENSION: &str = ".scheme.ltx";

pub(crate) const LTX_SYMBOL_COMMENT: char = ';';

pub(crate) const LTX_SYMBOL_INHERIT: char = ':';

pub(crate) const LTX_SYMBOL_INCLUDE: char = '#';

/// What makes an `#include` a mask over a directory rather than a name.
///
/// A statement carrying it expands to every config that matches, so its expansion is already the set that exists; one
/// without it expands to itself whether or not anything holds it. A reader that wants to know what an include reached
/// has to tell the two apart.
pub const LTX_SYMBOL_INCLUDE_WILDCARD: char = '*';

pub const LTX_SYMBOL_SCHEME: char = '$';

pub const LTX_SYMBOL_ANY: &str = "*";

pub(crate) const LTX_SYMBOL_SECTION_OPEN: char = '[';

pub(crate) const LTX_SYMBOL_SECTION_CLOSE: char = ']';

pub(crate) const LTX_SYMBOL_OPTIONAL: char = '?';

pub(crate) const LTX_SYMBOL_ARRAY: &str = "[]";

/// How an `Ltx` that was never read from a file names itself in a diagnostic.
pub(crate) const VIRTUAL_LTX_PATH: &str = "virtual";

/// The newline a canonical rewrite emits, which is the engine's own.
pub(crate) const LTX_LINE_SEPARATOR: &str = "\r\n";

/// What a canonical rewrite puts between a key and its value.
pub(crate) const DEFAULT_KV_SEPARATOR: &str = " = ";

/// What a canonical rewrite puts after a key whose value is empty: the separator without its trailing space.
pub(crate) const EMPTY_VALUE_KV_SEPARATOR: &str = " =";
