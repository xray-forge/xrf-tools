//! What a level's frame draws, one folder a group of passes, named for the part of the frame they draw. A group holds
//! its passes, built once for every view, its view stage where it has one, and a folder a feature: its unit, passes,
//! parameters, uniforms and the other types it owns. Beside the groups, `view` holds what a view's units share (the
//! `ViewFeature` trait, `SceneFeatures` and `ViewStages`), and `page` the units the page drives between frames: the
//! overlays, helpers and picks.

pub(crate) mod effects;
pub(crate) mod image;
pub(crate) mod lighting;
pub(crate) mod page;
pub(crate) mod scene;
pub(crate) mod tracing;
pub(crate) mod view;
pub(crate) mod weather;
