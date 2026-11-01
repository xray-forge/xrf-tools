use xrf_renderer_core::{PassParameters, StorageValueMut};

/// The grass sort's indirect dispatch, which `cs_arrange` alone writes.
#[derive(Clone, Copy, PassParameters)]
#[parameters(group = 3)]
pub struct GrassSortParameters {
  #[storage]
  pub sort_dispatch: StorageValueMut<[u32; 3]>,
}
