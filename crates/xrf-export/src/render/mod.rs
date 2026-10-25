//! Writing the manifest out: the formats it is published in and the renderers that lay each one out.

pub(crate) mod extern_format;
pub(crate) mod extern_html_renderer;
pub(crate) mod extern_xml_renderer;

#[cfg(test)]
mod tests;

pub use crate::render::extern_format::ExternFormat;
