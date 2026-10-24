//! A level map's annotations: the level's logic volumes, patrols and graph points drawn as an SVG over it.

mod annotation_canvas;
mod annotation_outline;
mod annotation_patrol;
mod annotation_volume;
mod level_annotation_kind;
mod level_annotations;
mod map_annotations_request;

pub use annotation_canvas::AnnotationCanvas;
pub use annotation_outline::{AnnotationOutline, AnnotationPoint};
pub use annotation_patrol::AnnotationPatrol;
pub use annotation_volume::AnnotationVolume;
pub use level_annotation_kind::LevelAnnotationKind;
pub use level_annotations::LevelAnnotations;
pub use map_annotations_request::MapAnnotationsRequest;
