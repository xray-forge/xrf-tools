//! Lightmaps: how a level's lightmapped faces are cut into charts and sized, as xrLC cuts and sizes them, lit and
//! compressed, packed onto lightmap pages, and the pages' textures composed; and the maps of the textures it lights
//! implicitly.

pub(crate) mod implicit_lighter;
pub(crate) mod implicit_map;
pub(crate) mod light_chart;
pub(crate) mod light_chart_lighter;
pub(crate) mod light_chart_mask;
pub(crate) mod light_chart_scene;
pub(crate) mod lightmap_atlas;
pub(crate) mod lightmap_faces;
pub(crate) mod lightmap_layer;
pub(crate) mod lightmap_page;
pub(crate) mod lightmap_placement;
pub(crate) mod lightmap_resampler;
pub(crate) mod lit_chart;
