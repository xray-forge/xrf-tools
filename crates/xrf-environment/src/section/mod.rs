//! One config section as authored: the key vocabulary every section kind is read with, its values as the engine reads
//! them, the keys it does not read, where each came from, and what the read found wrong.

pub(crate) mod environment_origin;
pub(crate) mod environment_section;
pub(crate) mod environment_section_reader;
pub(crate) mod finding;
pub(crate) mod key;

pub use crate::section::environment_origin::EnvironmentOrigin;
pub use crate::section::environment_section::EnvironmentSection;
pub(crate) use crate::section::environment_section_reader::EnvironmentSectionReader;
