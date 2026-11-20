//! How a light ray meets each face of the ray model: passed, blocked, or dimmed by its texture's alpha.

#[cfg(test)]
pub(crate) mod shadow_class;
#[cfg(test)]
pub(crate) mod shadow_class_counts;
pub(crate) mod shadow_face;
pub(crate) mod shadow_materials;
pub(crate) mod shadow_materials_request;
pub(crate) mod shadow_texture;
