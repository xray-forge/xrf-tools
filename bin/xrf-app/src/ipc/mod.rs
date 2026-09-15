//! The command surface the webview calls into, and everything derived from it.
//!
//! `registry` declares every command once and expands that list into runtime dispatch, ACL entries, and Specta
//! collections. `bindings` drives the same surface through `xrf-ipc-typescript` to write the frontend's
//! mirrors, and exists only in a test build because that is the harness it is run from.

#[cfg(all(test, feature = "typescript-bindings"))]
pub(crate) mod bindings;
pub(crate) mod registry;
