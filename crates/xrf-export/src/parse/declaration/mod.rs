//! Reading one source's `extern(...)` calls: the call itself, the value it publishes, a callable's contract, the
//! JSDoc above it, and the located errors any of them raises.

pub(crate) mod extern_callable_parser;
pub(crate) mod extern_declaration_parser;
pub(crate) mod extern_diagnostic;
pub(crate) mod extern_statement;
pub(crate) mod extern_type_renderer;
pub(crate) mod extern_value_parser;
pub(crate) mod js_doc_parser;

pub(crate) use crate::parse::declaration::extern_declaration_parser::ExternDeclarationParser;
pub(crate) use crate::parse::declaration::extern_statement::ExternStatement;
