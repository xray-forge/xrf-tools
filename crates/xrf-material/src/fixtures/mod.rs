//! Descriptor files and trees for tests, with the shapes this crate pins itself against.
//!
//! Shared with dependants through the `fixtures` feature so the sweep and the viewer are tested against one definition
//! of each declaration state rather than two hand-rolled byte layouts that can drift.

pub(crate) mod thm_fixture;
pub(crate) mod thm_fixture_tree;

pub use crate::fixtures::thm_fixture::ThmFixture;
pub use crate::fixtures::thm_fixture_tree::ThmFixtureTree;
