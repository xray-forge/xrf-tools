//! A view's stages, one for each part of its frame: each owns the units drawing in that part alone, takes their frame
//! in one `prepare`, and records them where the frame draws them, between the view-level units (`SceneFeatures`). A
//! stage reads the view's frame, the view-level units and what earlier stages answered, never another stage's unit,
//! which its fields keep private.

pub(crate) mod view_stages;
