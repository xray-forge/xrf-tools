//! The models a scene object's library object compiles to: what the game draws of it, each kind of surface it never
//! shows apart, and an occluder drawn as the Level Editor draws it.

mod scene_object_compiler;
mod scene_object_models;
mod scene_object_parts;
mod scene_occluder_look;
mod scene_surface_kind;
mod scene_surface_kinds;

pub(crate) use scene_object_compiler::SceneObjectCompiler;
pub(crate) use scene_object_models::SceneObjectModels;
pub(crate) use scene_object_parts::SceneObjectParts;
pub(crate) use scene_occluder_look::SceneOccluderLook;
pub(crate) use scene_surface_kind::SceneSurfaceKind;
pub(crate) use scene_surface_kinds::SceneSurfaceKinds;
