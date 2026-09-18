//! The compiled level of the X-Ray engine, one module per file the game ships.
//!
//! `level` is the render bundle itself and `geom` its geometry; the rest are the files beside them, each
//! named for the one it reads.

pub(crate) mod ai;
pub(crate) mod cform;
pub(crate) mod details;
pub(crate) mod env_mod;
pub(crate) mod fog_vol;
pub(crate) mod game;
pub(crate) mod geom;
pub(crate) mod hom;
pub(crate) mod level;
pub(crate) mod lights;
pub(crate) mod ps_static;
pub(crate) mod snd_static;
pub(crate) mod som;
pub(crate) mod wallmarks;

#[cfg(test)]
mod tests;

pub use crate::ai::level_ai_file::*;
pub use crate::cform::level_cform_file::*;
pub use crate::details::detail_model::*;
pub use crate::details::detail_vertex::*;
pub use crate::details::level_details_file::*;
pub use crate::details::level_details_slot::*;
pub use crate::env_mod::level_env_mod_file::*;
pub use crate::env_mod::level_env_modifier::*;
pub use crate::fog_vol::level_fog_vol_file::*;
pub use crate::fog_vol::level_fog_volume::*;
pub use crate::game::level_game_file::*;
pub use crate::game::level_game_rpoint::*;
pub use crate::game::level_game_way::*;
pub use crate::game::level_game_way_link::*;
pub use crate::game::level_game_way_point::*;
pub use crate::geom::level_geom_file::*;
pub use crate::geom::level_geom_index_buffer::*;
pub use crate::geom::level_geom_slide_window::*;
pub use crate::geom::level_geom_slide_window_item::*;
pub use crate::geom::level_geom_source::*;
pub use crate::geom::level_geom_vertex_buffer::*;
pub use crate::geom::level_geom_vertex_element::*;
pub use crate::geom::level_vertex::*;
pub use crate::geom::level_vertex_layout::*;
pub use crate::hom::level_hom_file::*;
pub use crate::hom::level_hom_polygon::*;
pub use crate::level::level_dynamic_light::*;
pub use crate::level::level_dynamic_lights_chunk::*;
pub use crate::level::level_file::*;
pub use crate::level::level_header_chunk::*;
pub use crate::level::level_light_color::*;
pub use crate::level::level_portal::*;
pub use crate::level::level_portals_chunk::*;
pub use crate::level::level_sector::*;
pub use crate::level::level_sectors_chunk::*;
pub use crate::level::level_shader_entry::*;
pub use crate::level::level_shaders_chunk::*;
pub use crate::level::level_visual::*;
pub use crate::level::level_visuals_chunk::*;
pub use crate::lights::level_light::*;
pub use crate::lights::level_lights_chunk::*;
pub use crate::lights::level_lights_file::*;
pub use crate::ps_static::level_ps_static_file::*;
pub use crate::ps_static::level_ps_static_placement::*;
pub use crate::snd_static::level_snd_static_file::*;
pub use crate::snd_static::level_snd_static_sound::*;
pub use crate::snd_static::level_snd_static_window::*;
pub use crate::som::level_som_file::*;
pub use crate::som::level_som_polygon::*;
pub use crate::wallmarks::level_wallmark::*;
pub use crate::wallmarks::level_wallmark_slot::*;
pub use crate::wallmarks::level_wallmark_vertex::*;
pub use crate::wallmarks::level_wallmarks_file::*;
