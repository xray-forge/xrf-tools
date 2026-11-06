use xrf_editable_object::{EditableMesh, EditableSurface};

/// A decompiled mesh and the surfaces its faces are made of.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct AssembledMesh {
  pub(crate) mesh: EditableMesh,
  pub(crate) surfaces: Vec<EditableSurface>,
}
