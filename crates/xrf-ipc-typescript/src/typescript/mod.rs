//! TypeScript as text and as a Specta format, knowing nothing of the IPC surface being mirrored.
//!
//! Every module here is a leaf: they are depended on from above and depend on nothing else in the crate, which is
//! what makes them vocabulary rather than a layer.

pub(crate) mod format;
pub(crate) mod normalization;
pub(crate) mod source;
pub(crate) mod syntax;
