//! The field types a pass's parameters bind buffers through, each knowing its WGSL type and its graph access.

mod storage_array;
mod storage_array_mut;
mod storage_field;
mod uniform_binding;
mod uniform_field;

pub use storage_array::StorageArray;
pub use storage_array_mut::StorageArrayMut;
pub use storage_field::StorageField;
pub use uniform_binding::UniformBinding;
pub use uniform_field::UniformField;
