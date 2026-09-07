//! Descriptor files and trees for tests, with the shapes this crate pins itself against.
//!
//! Shared with dependants through the `fixtures` feature so the sweep and the viewer are tested against one definition
//! of each declaration state rather than two hand-rolled byte layouts that can drift.

pub(crate) mod fixture_tree;
pub(crate) mod thm_fixture;

pub use crate::fixtures::fixture_tree::FixtureTree;
pub use crate::fixtures::thm_fixture::ThmFixture;
