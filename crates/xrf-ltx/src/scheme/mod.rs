//! What a section is allowed to hold, declared in a `*.scheme.ltx` file beside the configs it governs.

pub(crate) mod ltx_field_data_type;
pub(crate) mod ltx_field_scheme;
pub(crate) mod ltx_scheme_parser;
pub(crate) mod ltx_section_scheme;
pub(crate) mod tuple_separator;

pub use crate::scheme::ltx_field_data_type::LtxFieldDataType;
pub use crate::scheme::ltx_field_scheme::LtxFieldScheme;
pub use crate::scheme::ltx_scheme_parser::LtxSchemeParser;
pub use crate::scheme::ltx_section_scheme::LtxSectionScheme;
pub use crate::scheme::tuple_separator::TupleSeparator;
