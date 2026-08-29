pub const ROOT_SECTION: &str = "";

pub const LTX_EXTENSION: &str = "ltx";

pub const LTX_SCHEME_FIELD: &str = "$scheme";

pub const LTX_SCHEME_STRICT_FIELD: &str = "$strict";

pub const LTX_SCHEME_LTX_FILENAME: &str = "scheme.ltx";

/// Root config the engine loads, which every project entry point chain reaches.
pub const SYSTEM_LTX_FILENAME: &str = "system.ltx";

pub const LTX_SCHEME_EXTENSION: &str = ".scheme.ltx";

pub const LTX_SYMBOL_COMMENT: char = ';';

pub const LTX_SYMBOL_INHERIT: char = ':';

pub const LTX_SYMBOL_INCLUDE: char = '#';

pub const LTX_SYMBOL_SCHEME: char = '$';

pub const LTX_SYMBOL_ANY: &str = "*";

pub const LTX_SYMBOL_SECTION_OPEN: char = '[';

pub const LTX_SYMBOL_SECTION_CLOSE: char = ']';

pub const LTX_SYMBOL_OPTIONAL: char = '?';

pub const LTX_SYMBOL_ARRAY: &str = "[]";

/// How an `Ltx` that was never read from a file names itself in a diagnostic.
pub const VIRTUAL_LTX_PATH: &str = "virtual";
