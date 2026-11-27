//! Presentation as layers over the game: each takes the input the layers over it let through, moves on once a frame
//! and draws, and the chain (`GameLayers`) runs them in order.

pub(crate) mod game_input_fate;
pub(crate) mod game_layer;
pub(crate) mod game_layer_context;
pub(crate) mod game_layers;
pub(crate) mod game_player_layer;
