//! The element and attribute names a texture description file is written in.
//!
//! One vocabulary, in one place, because the reader and the writer have to agree on every one of them: a tag renamed
//! on one side and not the other produces a file the engine silently ignores rather than an error anybody sees.

/// Root node of a ui texture description file, wrapping all of its `file` nodes.
pub(crate) const XML_TAG_WINDOW: &str = "w";

pub(crate) const XML_TAG_FILE: &str = "file";

pub(crate) const XML_TAG_TEXTURE: &str = "texture";

pub(crate) const XML_ATTRIBUTE_ID: &str = "id";

pub(crate) const XML_ATTRIBUTE_NAME: &str = "name";

pub(crate) const XML_ATTRIBUTE_X: &str = "x";

pub(crate) const XML_ATTRIBUTE_Y: &str = "y";

pub(crate) const XML_ATTRIBUTE_WIDTH: &str = "width";

pub(crate) const XML_ATTRIBUTE_HEIGHT: &str = "height";
