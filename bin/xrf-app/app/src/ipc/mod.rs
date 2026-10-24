//! The command surface the webview calls into, and everything derived from it.
//!
//! `registry` composes what every plugin crate declares: the inline plugins and their permissions at build time, and the
//! transport routes at run time. `bindings` drives the same surface through `xrf-ipc-typescript` to write the frontend's
//! mirrors, and exists only in a test build because that is the harness it is run from.

#[cfg(all(test, feature = "typescript-bindings"))]
pub(crate) mod bindings;
pub(crate) mod registry;
