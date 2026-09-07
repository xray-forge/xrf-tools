//! Files for tests, with the shapes this crate pins itself against.
//!
//! Shared with dependants through the `fixtures` feature, so a crate resolving what a blender means is tested against
//! one definition of each class's property grid rather than a hand-rolled byte layout of its own.

pub(crate) mod shader_blender_fixture;
pub(crate) mod shader_library_fixture;

pub use crate::fixtures::shader_blender_fixture::ShaderBlenderFixture;
pub use crate::fixtures::shader_library_fixture::ShaderLibraryFixture;
