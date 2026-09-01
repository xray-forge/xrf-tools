//! What a translation run calls the work it is doing.

/// Phase a build reports while it compiles sources into string tables.
pub const TRANSLATION_PHASE_BUILD: &str = "build";

/// Phase a formatting run reports while it normalizes sources in place.
pub const TRANSLATION_PHASE_FORMAT: &str = "format";

/// Phase an import reports while it reads raw tables and writes sources.
pub const TRANSLATION_PHASE_PARSE: &str = "parse";

/// Phase a verification reports while it checks sources for missing translations.
pub const TRANSLATION_PHASE_VERIFY: &str = "verify";
