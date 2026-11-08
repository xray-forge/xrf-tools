//! Comparing two trees and publishing the difference as a patch volume.

pub(crate) mod commands;
mod request;

pub(crate) use request::ArchivesPatchRequest;
