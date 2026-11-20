use serde::Serialize;
use xrf_thm::{ThmFile, ThmGroupFile, ThmObjectFile, ThmSoundFile};

/// What `thm info` read out of a thumbnail: its kind, and the crate's own reading of it under `file`.
#[derive(Debug, Serialize)]
#[serde(tag = "kind", content = "file", rename_all = "camelCase")]
pub(crate) enum ThmInfoReport {
  Texture(ThmFile),
  Object(ThmObjectFile),
  Group(ThmGroupFile),
  Sound(ThmSoundFile),
}
