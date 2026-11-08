//! What the configs commands taking more than a few arguments were asked to do.

mod configs_find_declarations_request;
mod configs_format_request;
mod configs_list_request;
mod configs_open_request;
mod configs_read_document_request;
mod configs_read_section_scheme_request;
mod configs_read_sections_request;
mod configs_verify_request;

pub(crate) use configs_find_declarations_request::ConfigsFindDeclarationsRequest;
pub(crate) use configs_format_request::ConfigsFormatRequest;
pub(crate) use configs_list_request::ConfigsListRequest;
pub(crate) use configs_open_request::ConfigsOpenRequest;
pub(crate) use configs_read_document_request::ConfigsReadDocumentRequest;
pub(crate) use configs_read_section_scheme_request::ConfigsReadSectionSchemeRequest;
pub(crate) use configs_read_sections_request::ConfigsReadSectionsRequest;
pub(crate) use configs_verify_request::ConfigsVerifyRequest;
