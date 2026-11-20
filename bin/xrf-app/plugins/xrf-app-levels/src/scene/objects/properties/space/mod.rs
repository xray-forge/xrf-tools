//! Where a scene object stands and the volumes its shapes enclose.

mod scene_object_transform;
mod scene_shape;
mod scene_shape_element;
mod scene_shape_usage;

pub(crate) use scene_object_transform::SceneObjectTransform;
pub(crate) use scene_shape::SceneShape;
pub(crate) use scene_shape_element::SceneShapeElement;
pub(crate) use scene_shape_usage::SceneShapeUsage;
