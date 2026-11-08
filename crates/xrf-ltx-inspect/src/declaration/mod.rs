//! Where a section or a field is written, across every config a project holds.

pub(crate) mod ltx_declaration;
pub(crate) mod ltx_declaration_query;
pub(crate) mod ltx_declaration_reader;
pub(crate) mod ltx_declaration_search;
pub(crate) mod ltx_declaration_statement;
pub(crate) mod ltx_unreadable_config;

pub use crate::declaration::ltx_declaration::LtxDeclaration;
pub use crate::declaration::ltx_declaration_query::LtxDeclarationQuery;
pub use crate::declaration::ltx_declaration_reader::LtxDeclarationReader;
pub use crate::declaration::ltx_declaration_search::LtxDeclarationSearch;
pub use crate::declaration::ltx_declaration_statement::LtxDeclarationStatement;
pub use crate::declaration::ltx_unreadable_config::LtxUnreadableConfig;
