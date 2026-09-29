//! One config section as authored: its values as the engine reads them, the keys it does not read, and where each
//! came from.

pub(crate) mod environment_origin;
pub(crate) mod environment_section;
pub(crate) mod environment_section_reader;

pub use crate::section::environment_origin::EnvironmentOrigin;
pub use crate::section::environment_section::EnvironmentSection;
pub(crate) use crate::section::environment_section_reader::EnvironmentSectionReader;
