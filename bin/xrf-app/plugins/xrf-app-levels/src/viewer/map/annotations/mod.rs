//! A level map's annotations: the level objects of the kinds asked for drawn as an SVG over it.

mod annotation_canvas;
mod annotation_marker;
mod annotation_outline;
mod annotation_patrol;
mod annotation_point;
mod annotation_volume;
mod level_annotations;
mod map_annotations_write_request;

pub(crate) use annotation_canvas::AnnotationCanvas;
pub(crate) use annotation_marker::AnnotationMarker;
pub(crate) use annotation_outline::AnnotationOutline;
pub(crate) use annotation_patrol::AnnotationPatrol;
pub(crate) use annotation_point::AnnotationPoint;
pub(crate) use annotation_volume::AnnotationVolume;
pub(crate) use level_annotations::LevelAnnotations;
pub(crate) use map_annotations_write_request::MapAnnotationsWriteRequest;
