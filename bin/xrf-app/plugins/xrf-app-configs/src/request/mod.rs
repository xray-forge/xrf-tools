//! What the configs commands taking more than a few arguments were asked to do.

mod configs_find_declarations_request;
mod configs_format_request;
mod configs_list_request;
mod configs_open_request;
mod configs_read_document_request;
mod configs_read_section_scheme_request;
mod configs_read_sections_request;
mod configs_verify_request;

pub use configs_find_declarations_request::ConfigsFindDeclarationsRequest;
pub use configs_format_request::ConfigsFormatRequest;
pub use configs_list_request::ConfigsListRequest;
pub use configs_open_request::ConfigsOpenRequest;
pub use configs_read_document_request::ConfigsReadDocumentRequest;
pub use configs_read_section_scheme_request::ConfigsReadSectionSchemeRequest;
pub use configs_read_sections_request::ConfigsReadSectionsRequest;
pub use configs_verify_request::ConfigsVerifyRequest;
