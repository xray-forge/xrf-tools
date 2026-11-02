mod generated_shaders;
pub(crate) mod gpu_probe;
mod shader_composer;
mod shader_mirrors;
mod shader_permutations;
// The watcher watches only in a debug build: a release build's never sees a change.
#[cfg(debug_assertions)]
mod shader_watcher;
mod shaders;
pub(crate) mod wgsl_probe;
