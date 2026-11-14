//! What each scene object's kind keeps beyond its name and transform, as the page is shown it.

mod scene_light_type;
mod scene_object_properties;
mod scene_sector_item;
mod scene_spawn_field;
mod scene_spawn_kind;
mod scene_way_link;
mod scene_way_point;
mod scene_way_type;
mod space;

pub(crate) use scene_light_type::SceneLightType;
pub(crate) use scene_object_properties::SceneObjectProperties;
pub(crate) use scene_sector_item::SceneSectorItem;
pub(crate) use scene_spawn_field::SceneSpawnField;
pub(crate) use scene_spawn_kind::SceneSpawnKind;
pub(crate) use scene_way_link::SceneWayLink;
pub(crate) use scene_way_point::SceneWayPoint;
pub(crate) use scene_way_type::SceneWayType;
pub(crate) use space::{SceneObjectTransform, SceneShape, SceneShapeElement, SceneShapeUsage};
