//! This application's frontend mirrors: what goes into them, and the check that they still describe it.
//!
//! The generator is the `xrf-ipc-typescript` crate, which knows nothing of this application. What lives here is the
//! part that does — which Specta builders make up the surface, and where the output goes.

mod generator;
