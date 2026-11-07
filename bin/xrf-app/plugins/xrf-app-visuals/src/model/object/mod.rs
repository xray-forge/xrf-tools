//! An SDK `.object` opened in the viewer: compiled in memory and described for what only it holds.

mod selected_object;
mod selected_object_bone;
mod selected_object_description;
mod selected_object_provenance;
mod selected_object_surface;

pub use selected_object::SelectedObject;
pub use selected_object_bone::SelectedObjectBone;
pub use selected_object_description::SelectedObjectDescription;
pub use selected_object_provenance::SelectedObjectProvenance;
pub use selected_object_surface::SelectedObjectSurface;
