//! Which of this application's surfaces reach the frontend, and the check that its mirrors still describe them.
//!
//! The generator itself is `xrf-ipc-typescript`, which knows nothing of this application: what lives here is the list
//! of Specta builders to export and the output they are written to, in `bin/xrf-ui/src/core/ipc/`. Regenerate with
//! `cargo make generate-typescript`.

#[cfg(test)]
mod tests;
