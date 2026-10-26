//! The chunks of a visual, grouped by the engine layer that reads them: every visual's own (`visual`), a standalone
//! mesh's embedded parts (`mesh`), a skeleton's (`skeleton`), and those only a level visual carries (`level`).

pub(crate) mod level;
pub(crate) mod mesh;
pub(crate) mod skeleton;
pub(crate) mod visual;
