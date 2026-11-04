//! The level viewer's own files in the application's data: the views it exports, the screenshots and captures it
//! takes of a viewport, a level map's and a cubemap's exported again, the bursts of frames it measures, and the videos
//! it writes of them.

pub(crate) mod burst;
mod capture;
pub(crate) mod commands;
pub(crate) mod cube;
mod folder;
pub(crate) mod map;
pub(crate) mod record;
pub(crate) mod routes;

pub use capture::{
  LevelViewerReport, ScreenshotSaveRequest, ViewerAssembly, ViewerCaptureJob, ViewerDdsExport, ViewerImage,
};
pub use folder::LevelViewerFolder;
